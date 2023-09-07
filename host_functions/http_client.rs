use core::ptr::NonNull;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use js::c;
use qjsbind as js;

pub fn setup(pink: &js::Value) -> js::Result<()> {
    pink.define_property_fn("httpRequest", http_request)?;
    pink.define_property_fn("batchHttpRequest", host_batch_http_request)?;
    Ok(())
}

enum Either<A, B> {
    A(A),
    B(B),
}

impl<A, B> js::ToJsValue for Either<A, B>
where
    A: js::ToJsValue,
    B: js::ToJsValue,
{
    fn to_js_value(&self, ctx: NonNull<c::JSContext>) -> js::Result<js::Value> {
        match &self {
            Either::A(a) => a.to_js_value(ctx),
            Either::B(b) => b.to_js_value(ctx),
        }
    }
}

#[derive(js::FromJsValue)]
#[qjsbind(rename_all = "camelCase")]
struct HttpRequest {
    url: String,
    #[qjsbind(default = "defualt_method")]
    method: String,
    #[qjsbind(default)]
    headers: Headers,
    #[qjsbind(default, as_bytes)]
    body: Vec<u8>,
    #[qjsbind(default)]
    return_text_body: bool,
}
fn defualt_method() -> String {
    "GET".to_string()
}

impl From<HttpRequest> for pink::chain_extension::HttpRequest {
    fn from(req: HttpRequest) -> Self {
        Self {
            url: req.url,
            method: req.method,
            headers: req.headers.pairs,
            body: req.body,
        }
    }
}

#[derive(js::ToJsValue)]
#[qjsbind(rename_all = "camelCase")]
struct HttpResponse {
    status_code: u16,
    reason_phrase: String,
    headers: Vec<(String, String)>,
    body: Either<String, js::AsBytes<Vec<u8>>>,
}

#[derive(js::ToJsValue)]
struct HttpError {
    error: String,
}

#[js::host_call]
fn http_request(req: HttpRequest) -> Result<HttpResponse, String> {
    let return_text_body = req.return_text_body;
    let response = pink::ext().http_request(req.into());
    let body = if return_text_body {
        let body = String::from_utf8_lossy(&response.body);
        Either::A(body.into())
    } else {
        Either::B(js::AsBytes(response.body))
    };
    Ok(HttpResponse {
        status_code: response.status_code,
        reason_phrase: response.reason_phrase,
        headers: response.headers,
        body,
    })
}

#[js::host_call]
fn host_batch_http_request(
    requests: Vec<HttpRequest>,
    timeout_ms: Option<u64>,
) -> Result<Vec<Either<HttpResponse, HttpError>>, String> {
    let return_text_bodies = requests
        .iter()
        .map(|r| r.return_text_body)
        .collect::<Vec<_>>();
    let responses = pink::ext()
        .batch_http_request(
            requests.into_iter().map(Into::into).collect(),
            timeout_ms.unwrap_or(10),
        )
        .map_err(|err| alloc::format!("Failed to call batch_http_request: {err:?}"))?;

    if responses.len() != return_text_bodies.len() {
        return Err("Mismatch between number of responses and returnTextBody flags".into());
    }

    let mut response_objects = Vec::new();
    for (response, return_text_body) in responses.into_iter().zip(return_text_bodies.into_iter()) {
        match response {
            Ok(response) => {
                let pink::chain_extension::HttpResponse {
                    status_code,
                    reason_phrase,
                    headers,
                    body,
                } = response;
                let body = if return_text_body {
                    Either::A(String::from_utf8_lossy(&body).into())
                } else {
                    Either::B(js::AsBytes(body))
                };
                response_objects.push(Either::A(HttpResponse {
                    status_code,
                    reason_phrase,
                    headers,
                    body,
                }))
            }
            Err(err) => response_objects.push(Either::B(HttpError {
                error: alloc::format!("{:?}", err),
            })),
        }
    }
    Ok(response_objects)
}

#[derive(Debug, Default)]
pub struct Headers {
    pairs: Vec<(String, String)>,
}

impl js::FromJsValue for Headers {
    fn from_js_value(value: js::Value) -> js::Result<Self> {
        Ok(if value.is_array() {
            Vec::<(String, String)>::from_js_value(value)?.into()
        } else {
            BTreeMap::<String, String>::from_js_value(value)?.into()
        })
    }
}

impl js::ToJsValue for Headers {
    fn to_js_value(&self, ctx: NonNull<c::JSContext>) -> js::Result<js::Value> {
        self.pairs.to_js_value(ctx)
    }
}

impl From<Vec<(String, String)>> for Headers {
    fn from(pairs: Vec<(String, String)>) -> Self {
        Self { pairs }
    }
}

impl From<BTreeMap<String, String>> for Headers {
    fn from(headers: BTreeMap<String, String>) -> Self {
        Self {
            pairs: headers.into_iter().collect(),
        }
    }
}

impl From<Headers> for Vec<(String, String)> {
    fn from(headers: Headers) -> Self {
        headers.pairs
    }
}

impl FromIterator<(String, String)> for Headers {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        Self {
            pairs: iter.into_iter().collect(),
        }
    }
}
