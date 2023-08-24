use std::collections::BTreeMap;

use log::info;
use qjs::{host_call, FromJsValue, ToJsValue, Value as JsValue};

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

#[derive(FromJsValue, Debug)]
#[qjsbind(rename_all = "camelCase")]
struct HttpResponseHead {
    status: u16,
    headers: BTreeMap<String, String>,
}

#[derive(ToJsValue, Debug)]
struct Event<'a, Data> {
    name: &'a str,
    data: Data,
}

pub fn setup(ns: &JsValue) -> Result<()> {
    ns.set_property_fn("httpListen", http_listen)?;
    ns.set_property_fn("httpSendResponse", http_send_response)?;
    Ok(())
}

#[host_call]
fn http_listen(service: ServiceRef, _this: JsValue, callback: OwnedJsValue) {
    service.set_http_listener(callback)
}

fn use_2nd<F1, I1, F2, O>(_f1: F1, f2: F2) -> Option<O>
where
    F1: FnOnce(I1) -> O,
    F2: FnOnce() -> Option<O>,
{
    f2()
}

#[host_call]
fn http_send_response(_service: ServiceRef, _this: JsValue, tx: JsValue, response: HttpResponseHead) {
    let Some(response_tx) = use_2nd(
        |req: crate::runtime::HttpRequest| req.response_tx,
        || tx.opaque_object_take_data(),
    ) else {
        info!("Failed to get response tx");
        return;
    };
    if let Err(err) = response_tx.send(crate::runtime::HttpResponseHead {
        status: response.status,
        headers: response.headers.into_iter().collect(),
    }) {
        info!("Failed to send response: {err:?}");
    }
}

pub(crate) fn try_accept_http_request(
    service: ServiceRef,
    request: crate::runtime::HttpRequest,
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
