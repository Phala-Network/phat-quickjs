use anyhow::Context;
use hyper::{body::HttpBody, Body};
use log::info;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

use crate::{
    runtime::{http_connector, time::timeout, HyperExecutor},
    service::OwnedJsValue,
};
use qjs::{host_call, AsBytes, Error as ValueError, FromJsValue, ToJsValue, Value as JsValue};

use super::*;

#[derive(Debug, Default)]
pub struct Headers {
    pairs: Vec<(String, String)>,
}

impl FromJsValue for Headers {
    fn from_js_value(value: JsValue) -> Result<Self, ValueError> {
        Ok(if value.is_array() {
            Vec::<(String, String)>::from_js_value(value)?.into()
        } else {
            BTreeMap::<String, String>::from_js_value(value)?.into()
        })
    }
}

impl ToJsValue for Headers {
    fn to_js_value(&self, ctx: NonNull<c::JSContext>) -> Result<JsValue, ValueError> {
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

#[derive(FromJsValue, Debug)]
#[qjsbind(rename_all = "camelCase")]
pub struct HttpRequest {
    url: String,
    #[qjsbind(default = "default_method")]
    method: String,
    #[qjsbind(default)]
    headers: Headers,
    #[qjsbind(default, as_bytes)]
    body: Vec<u8>,
    body_text: Option<String>,
    #[qjsbind(default = "default_timeout")]
    timeout_ms: u64,
}

#[derive(ToJsValue, Debug)]
#[qjsbind(rename_all = "camelCase")]
struct HttpResponseHead {
    status: u16,
    status_text: String,
    version: String,
    headers: Headers,
}

#[derive(ToJsValue, Debug)]
struct Event<'a, Data> {
    name: &'a str,
    data: Data,
}

pub fn setup(ns: &JsValue) -> Result<()> {
    ns.define_property_fn("httpRequest", http_request)?;
    Ok(())
}

#[host_call(with_context)]
fn http_request(
    service: ServiceRef,
    _this: JsValue,
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
    let result = do_http_request_inner(weak_service.clone(), id, req).await;
    if let Err(err) = result {
        invoke_callback(&weak_service, id, "error", err.to_string());
    }
}
async fn do_http_request_inner(
    weak_service: ServiceWeakRef,
    id: u64,
    req: HttpRequest,
) -> Result<()> {
    let connector = http_connector();
    let client = hyper::Client::builder()
        .executor(HyperExecutor)
        .build::<_, Body>(connector);
    let uri: hyper::Uri = req
        .url
        .parse()
        .with_context(|| format!("Failed to parse url: {}", req.url))?;
    let mut builder = hyper::Request::builder()
        .method(req.method.as_str())
        .uri(&uri);
    let body: Vec<u8> = if let Some(body_text) = req.body_text {
        body_text.into_bytes()
    } else {
        req.body
    };
    for (k, v) in req.headers.pairs.iter() {
        builder = builder.header(k.as_str(), v.as_str());
    }
    let headers: BTreeSet<&str> = req.headers.pairs.iter().map(|(k, _v)| k.as_str()).collect();
    // Append Host, Content-Length and Content-Length if not present
    if !headers.contains("Host") {
        builder = builder.header("Host", uri.host().unwrap_or_default());
    }
    if !headers.contains("Content-Length") {
        builder = builder.header("Content-Length", body.len());
    }
    if !headers.contains("User-Agent") {
        builder = builder.header("User-Agent", "sidevm-quickjs/0.1.0");
    }
    let request = builder
        .body(Body::from(body))
        .context("Failed to build request")?;
    let response = timeout(
        Duration::from_millis(req.timeout_ms),
        client.request(request),
    )
    .await
    .context("Failed to send request: Timed out")?
    .context("Failed to send request")?;
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
        invoke_callback(&weak_service, id, "head", head);
    }
    tokio::pin!(response);
    while let Some(chunk) = response.data().await {
        let chunk = chunk.context("Failed to read response body")?;
        invoke_callback(&weak_service, id, "data", AsBytes(chunk));
    }
    invoke_callback(&weak_service, id, "end", ());
    Ok(())
}

fn invoke_callback(weak_service: &Weak<Service>, id: u64, name: &str, data: impl ToJsValue) {
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
