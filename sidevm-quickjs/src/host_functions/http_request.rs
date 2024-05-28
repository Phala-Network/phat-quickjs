use anyhow::{anyhow, Context};
use log::info;
use std::{collections::BTreeMap, time::Duration};

use crate::{runtime::time::sleep, service::OwnedJsValue};
use js::{AsBytes, Error as ValueError, FromJsValue, ToJsValue};

use super::*;

#[derive(Debug, Default)]
pub struct Pairs {
    pub(crate) pairs: Vec<(String, String)>,
}

impl FromJsValue for Pairs {
    fn from_js_value(value: js::Value) -> Result<Self, ValueError> {
        Ok(if value.is_array() {
            Vec::<(String, String)>::from_js_value(value)?.into()
        } else {
            BTreeMap::<String, String>::from_js_value(value)?.into()
        })
    }
}

impl ToJsValue for Pairs {
    fn to_js_value(&self, ctx: &js::Context) -> Result<js::Value, ValueError> {
        self.pairs.to_js_value(ctx)
    }
}

impl From<Vec<(String, String)>> for Pairs {
    fn from(pairs: Vec<(String, String)>) -> Self {
        Self { pairs }
    }
}

impl From<BTreeMap<String, String>> for Pairs {
    fn from(headers: BTreeMap<String, String>) -> Self {
        Self {
            pairs: headers.into_iter().collect(),
        }
    }
}

impl From<Pairs> for Vec<(String, String)> {
    fn from(headers: Pairs) -> Self {
        headers.pairs
    }
}

impl FromIterator<(String, String)> for Pairs {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        Self {
            pairs: iter.into_iter().collect(),
        }
    }
}

#[derive(FromJsValue, Debug)]
#[qjs(rename_all = "camelCase")]
pub struct HttpRequest {
    url: String,
    #[qjs(default = "default_method")]
    method: String,
    #[qjs(default)]
    headers: Pairs,
    #[qjs(default)]
    body: js::BytesOrString,
    #[qjs(default = "default_timeout")]
    timeout_ms: u64,
}

#[derive(ToJsValue, Debug)]
#[qjs(rename_all = "camelCase")]
struct HttpResponseHead {
    status: u16,
    status_text: String,
    version: String,
    headers: Pairs,
}

#[derive(ToJsValue, Debug)]
struct Event<'a, Data> {
    name: &'a str,
    data: Data,
}

pub fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("httpRequest", http_request)?;
    Ok(())
}

#[js::host_call(with_context)]
fn http_request(
    service: ServiceRef,
    _this: js::Value,
    req: HttpRequest,
    callback: OwnedJsValue,
) -> Result<u64> {
    Ok(service.spawn(callback, do_http_request, req))
}

fn default_method() -> String {
    "GET".into()
}

fn default_timeout() -> u64 {
    30_000
}

async fn do_http_request(weak_service: ServiceWeakRef, id: u64, req: HttpRequest) {
    let url = req.url.clone();
    let result = tokio::select! {
        _ = sleep(Duration::from_millis(req.timeout_ms)) => {
            Err(anyhow!("Timed out"))
        }
        result = do_http_request_inner(weak_service.clone(), id, req) => result,
    };
    if let Err(err) = result {
        invoke_callback(
            &weak_service,
            id,
            "error",
            &format!("Failed to request `{url}`: {err:?}"),
        );
    }
}

#[cfg(not(feature = "web"))]
async fn do_http_request_inner(
    weak_service: ServiceWeakRef,
    id: u64,
    req: HttpRequest,
) -> Result<()> {
    use crate::runtime::{http_connector, HyperExecutor};
    use core::pin::pin;
    use hyper::{body::HttpBody, Body};
    let connector = http_connector();
    let client = hyper::Client::builder()
        .executor(HyperExecutor)
        .build::<_, Body>(connector);
    let uri: hyper::Uri = req
        .url
        .parse()
        .with_context(|| format!("Failed to parse url: {}", req.url))?;
    let mut builder = hyper::Request::builder()
        .method(req.method.to_uppercase().as_str())
        .uri(&uri);
    let body = req.body.as_bytes().to_vec();
    for (k, v) in req.headers.pairs.iter() {
        builder = builder.header(k.as_str(), v.as_str());
    }
    let headers_map = builder
        .headers_mut()
        .ok_or_else(|| anyhow!("Failed to build request"))?;
    // Append Host, Content-Length and User-Agent if not present
    if !headers_map.contains_key("Host") {
        headers_map.insert("Host", uri.host().unwrap_or_default().parse()?);
    }
    if !headers_map.contains_key("Content-Length") {
        headers_map.insert("Content-Length", body.len().to_string().parse()?);
    }
    if !headers_map.contains_key("User-Agent") {
        headers_map.insert("User-Agent", "PhatContract/0.1.0".parse()?);
    }
    let request = builder
        .body(Body::from(body))
        .context("Failed to build request")?;
    let response = client.request(request).await?;
    {
        let head = {
            let headers = response
                .headers()
                .iter()
                .map(|(k, v)| (k.as_str().into(), v.to_str().unwrap_or_default().into()))
                .collect();
            let status = response.status().as_u16();
            let status_text = response
                .status()
                .canonical_reason()
                .unwrap_or_default()
                .into();
            let version = format!("{:?}", response.version());
            HttpResponseHead {
                status,
                status_text,
                version,
                headers,
            }
        };
        invoke_callback(&weak_service, id, "head", &head);
    }
    let mut response = pin!(response);
    while let Some(chunk) = response.data().await {
        let chunk = chunk.context("Failed to read response body")?;
        invoke_callback(&weak_service, id, "data", &AsBytes(chunk));
    }
    invoke_callback(&weak_service, id, "end", &());
    Ok(())
}

#[cfg(feature = "web")]
async fn do_http_request_inner(
    weak_service: ServiceWeakRef,
    id: u64,
    req: HttpRequest,
) -> Result<()> {
    use reqwest::{Client, Method};
    let method = Method::from_bytes(req.method.as_bytes()).context("Invalid method")?;
    let mut builder = Client::new().request(method, req.url);
    let mut has_ua = false;
    for (k, v) in req.headers.pairs.iter() {
        if k.eq_ignore_ascii_case("User-Agent") {
            has_ua = true;
        }
        builder = builder.header(k, v);
    }
    // Append User-Agent if not present
    if !has_ua {
        builder = builder.header("User-Agent", "PhatContract/0.1.0");
    }
    let body = req.body.as_bytes().to_vec();
    builder = builder.body(body);
    let response = builder.send().await?;
    let head = {
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.as_str().into(), v.to_str().unwrap_or_default().into()))
            .collect();
        let status = response.status().as_u16();
        let status_text = response
            .status()
            .canonical_reason()
            .unwrap_or_default()
            .into();
        HttpResponseHead {
            status,
            status_text,
            version: "HTTP/1.1".into(),
            headers,
        }
    };
    invoke_callback(&weak_service, id, "head", &head);
    let body = response.bytes().await?;
    invoke_callback(&weak_service, id, "data", &AsBytes(body));
    invoke_callback(&weak_service, id, "end", &());
    Ok(())
}

fn invoke_callback(weak_service: &Weak<Service>, id: u64, name: &str, data: &dyn ToJsValue) {
    let Some(service) = weak_service.upgrade() else {
        info!("http_request {id} exited because the service has been dropped");
        return;
    };
    let Some(callback) = service.get_resource_value(id) else {
        info!("http_request {id} exited because the resource has been dropped");
        return;
    };
    if let Err(err) = service.call_function(callback, (name, data)) {
        error!("[{id}] Failed to report http_request event {name}: {err:?}");
    }
}
