use anyhow::Context;
use hyper::{body::HttpBody, Body};
use log::info;
use qjs_sys::convert::{
    js_object_get_field, js_object_get_field_or_default, js_object_get_option_field,
};
use std::{collections::BTreeMap, time::Duration};

use crate::{
    runtime::{time::timeout, HttpConnector, HyperExecutor},
    service::OwnedJsValue,
};

use super::*;

struct HttpRequest {
    url: String,
    method: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
    timeout_ms: u64,
}

pub(super) fn http_request(
    service: ServiceRef,
    ctx: *mut c::JSContext,
    args: &[c::JSValueConst],
) -> Result<JsValue> {
    let Some(config) = args.get(0) else {
        anyhow::bail!("Invoking http_request without arguments");
    };

    let url: String = js_object_get_field(ctx, *config, "url").anyhow()?;
    let method: String = js_object_get_option_field(ctx, *config, "method")
        .anyhow()?
        .unwrap_or_else(|| "GET".into());
    let headers: BTreeMap<String, String> =
        js_object_get_field_or_default(ctx, *config, "headers").anyhow()?;
    let body: Vec<u8> = js_object_get_field_or_default(ctx, *config, "body").anyhow()?;
    let timeout_ms: u64 = js_object_get_field_or_default(ctx, *config, "timeout").anyhow()?;
    let callback: OwnedJsValue = js_object_get_field(ctx, *config, "callback").anyhow()?;
    let request = HttpRequest {
        url,
        method,
        headers,
        body,
        timeout_ms,
    };
    let id = service.spawn(callback, do_http_request, request);
    Ok(JsValue::Int(id as i32))
}

async fn do_http_request(weak_service: ServiceWeakRef, id: u64, req: HttpRequest) {
    let result = do_http_request_inner(weak_service.clone(), id, req).await;
    if let Err(err) = result {
        callback(
            &weak_service,
            id,
            "error",
            JsValue::String(err.to_string().into()),
        );
    }
}
async fn do_http_request_inner(
    weak_service: ServiceWeakRef,
    id: u64,
    req: HttpRequest,
) -> Result<()> {
    let connector = HttpConnector::new();
    let client = hyper::Client::builder()
        .executor(HyperExecutor)
        .build::<_, Body>(connector);
    let mut builder = hyper::Request::builder()
        .method(req.method.as_str())
        .uri(req.url);
    for (k, v) in req.headers {
        builder = builder.header(k.as_str(), v.as_str());
    }
    let request = builder
        .body(Body::from(req.body))
        .context("Failed to build request")?;
    let response = timeout(
        Duration::from_millis(req.timeout_ms),
        client.request(request),
    )
    .await
    .context("Failed to send request")??;
    {
        let head: JsValue = {
            let headers = BTreeMap::from_iter(response.headers().iter().map(|(k, v)| {
                (
                    k.as_str().into(),
                    JsValue::String(v.to_str().unwrap_or_default().into()),
                )
            }));
            let status = response.status().as_u16() as i32;
            let reason = response.status().canonical_reason().unwrap_or_default();
            let version = format!("{:?}", response.version());
            let response = BTreeMap::from_iter(vec![
                ("status".into(), JsValue::Int(status)),
                ("statusText".into(), JsValue::String(reason.into())),
                ("version".into(), JsValue::String(version)),
                ("headers".into(), JsValue::Object(headers)),
            ]);
            JsValue::Object(response)
        };
        callback(&weak_service, id, "head", head);
    }
    tokio::pin!(response);
    let todo = "Control chunk size";
    while let Some(chunk) = response.data().await {
        let chunk = chunk.context("Failed to read response body")?;
        callback(&weak_service, id, "data", JsValue::Bytes(chunk.into()));
    }
    callback(&weak_service, id, "end", JsValue::Null);
    Ok(())
}

fn callback(weak_service: &Weak<Service>, id: u64, name: &str, result: JsValue) {
    let Some(service) = weak_service.upgrade() else {
        info!("http_request {id} exited because the service is dropped");
        return;
    };
    let Some(res) = service.get_resource_value(id) else {
        info!("http_request {id} exited because the resource is dropped");
        return;
    };
    let ctx = service.raw_ctx();
    let args = vec![JsValue::String(name.into()), result]
        .into_iter()
        .map(|v| serialize_value(ctx, v))
        .collect::<Vec<_>>();

    let n_args = args.len();
    let args = args.into_iter().filter_map(|x| x.ok()).collect::<Vec<_>>();
    scopeguard::defer! {
        js_free_all(ctx, &args);
    }
    if n_args != args.len() {
        error!("[{id}] Failed to make args for http_request event {name}");
        return;
    }
    if let Err(err) = service.call_function(*res.value(), &args) {
        error!("[{id}] Failed to report http_request event {name}: {err}");
    }
}

fn js_free_all(ctx: *mut c::JSContext, args: &[c::JSValue]) {
    for arg in args {
        unsafe {
            c::JS_FreeValue(ctx, *arg);
        }
    }
}
