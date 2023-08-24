use std::collections::BTreeMap;

use qjs::{host_call, ToJsValue, Value as JsValue};

use super::*;
use crate::service::OwnedJsValue;

#[derive(ToJsValue, Debug)]
#[qjsbind(rename_all = "camelCase")]
pub struct HttpRequest {
    method: String,
    path: String,
    query: String,
    headers: BTreeMap<String, String>,
    opaque_io_stream: JsValue,
    opaque_response_tx: JsValue,
}

#[derive(ToJsValue, Debug)]
#[qjsbind(rename_all = "camelCase")]
struct HttpResponseHead {
    status: u16,
    status_text: String,
    version: String,
    headers: BTreeMap<String, String>,
}

#[derive(ToJsValue, Debug)]
struct Event<'a, Data> {
    name: &'a str,
    data: Data,
}

pub fn setup(ns: &JsValue) -> Result<()> {
    ns.set_property_fn("httpListen", http_listen)?;
    Ok(())
}

#[host_call]
fn http_listen(service: ServiceRef, _this: JsValue, callback: OwnedJsValue) {
    service.set_http_listener(callback)
}

pub(crate) fn try_accept_http_request(
    service: ServiceRef,
    request: sidevm::channel::HttpRequest,
) -> Result<()> {
    let Some(Ok(listener)) = service.http_listener().map(TryInto::try_into) else {
        return Ok(());
    };
    let req = HttpRequest {
        method: request.head.method.clone(),
        path: request.head.path.clone(),
        query: request.head.query.clone(),
        headers: request.head.headers.iter().cloned().collect(),
        opaque_io_stream: JsValue::new_opaque_object(service.raw_ctx(), request.io_stream),
        opaque_response_tx: JsValue::new_opaque_object(service.raw_ctx(), request.response_tx),
    };
    if let Err(err) = service.call_function(listener, (req,)) {
        anyhow::bail!("Failed to fire http request event: {err}");
    }
    Ok(())
}
