use js::{FromJsValue, ToJsValue};
use log::info;

use super::http_request::Headers;
use super::*;
use crate::service::OwnedJsValue;

#[derive(ToJsValue, Debug)]
#[qjsbind(rename_all = "camelCase")]
pub struct HttpRequest {
    method: String,
    url: String,
    headers: Headers,
    opaque_response_tx: js::Value,
    opaque_input_stream: js::Value,
    opaque_output_stream: js::Value,
}

#[derive(FromJsValue, Debug)]
#[qjsbind(rename_all = "camelCase")]
struct HttpResponseHead {
    status: u16,
    headers: Headers,
}

pub fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("httpListen", http_listen)?;
    ns.define_property_fn("httpSendResponseHead", http_send_response_head)?;
    Ok(())
}

#[js::host_call(with_context)]
fn http_listen(service: ServiceRef, _this: js::Value, callback: OwnedJsValue) {
    service.set_http_listener(callback)
}

pub(crate) fn try_accept_http_request(
    service: ServiceRef,
    request: crate::runtime::HttpRequest,
) -> Result<()> {
    let Some(Ok(listener)) = service.http_listener().map(TryInto::try_into) else {
        return Ok(());
    };
    let (input_stream, output_stream) = tokio::io::split(request.io_stream);
    let req = HttpRequest {
        method: request.head.method.clone(),
        url: request.head.url.clone(),
        headers: request.head.headers.iter().cloned().collect(),
        opaque_response_tx: js::Value::new_opaque_object(service.context(), request.response_tx),
        opaque_input_stream: js::Value::new_opaque_object(service.context(), input_stream),
        opaque_output_stream: js::Value::new_opaque_object(service.context(), output_stream),
    };
    if let Err(err) = service.call_function(listener, (req,)) {
        anyhow::bail!("failed to fire http request event: {err}");
    }
    Ok(())
}

#[js::host_call]
fn http_send_response_head(tx: js::Value, response: HttpResponseHead) {
    let Some(response_tx) = super::valueof_f2_as_typeof_f1(
        |req: crate::runtime::HttpRequest| req.response_tx,
        || tx.opaque_object_take_data(),
    ) else {
        info!("failed to get response tx");
        return;
    };
    if let Err(err) = response_tx.send(crate::runtime::HttpResponseHead {
        status: response.status,
        headers: response.headers.into(),
    }) {
        info!("failed to send response: {err:?}");
    }
}
