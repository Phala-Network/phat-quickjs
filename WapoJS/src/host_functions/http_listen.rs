use js::{FromJsValue, ToJsValue};
use log::{debug, info, trace};

use super::http_request::Headers;
use super::*;
use crate::service::OwnedJsValue;

#[derive(ToJsValue, Debug)]
#[qjs(rename_all = "camelCase")]
pub struct HttpRequest {
    method: String,
    url: String,
    headers: Headers,
    opaque_response_tx: js::Value,
    opaque_input_stream: js::Value,
    opaque_output_stream: js::Value,
}

#[derive(FromJsValue, Debug)]
#[qjs(rename_all = "camelCase")]
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

#[cfg(feature = "js-http-listen")]
#[allow(dead_code)]
pub(crate) fn try_accept_http_request(
    service: ServiceRef,
    request: crate::runtime::HttpRequest,
) -> Result<()> {
    let Some(Ok(listener)) = service.http_listener().map(TryInto::try_into) else {
        debug!(target: "js::https", "no http listener, ignoring request");
        return Ok(());
    };
    trace!(target: "js::https", "accepting http request: {:#?}", request.head);
    let (input_stream, output_stream) = tokio::io::split(request.io_stream);
    let req = HttpRequest {
        method: request.head.method.clone(),
        url: request.head.url.clone(),
        headers: request.head.headers.iter().cloned().collect(),
        opaque_response_tx: js::Value::new_opaque_object(
            service.context(),
            Some("HttpHeadTx"),
            request.response_tx,
        ),
        opaque_input_stream: js::Value::new_opaque_object(
            service.context(),
            Some("HttpInputBodyStream"),
            input_stream,
        ),
        opaque_output_stream: js::Value::new_opaque_object(
            service.context(),
            Some("HttpOutputBodyStream"),
            output_stream,
        ),
    };
    if let Err(err) = service.call_function(listener, (req,)) {
        anyhow::bail!("failed to fire http request event: {err}");
    }
    Ok(())
}

#[js::host_call]
fn http_send_response_head(tx: js::Value, response: HttpResponseHead) {
    trace!(target: "js::https", "sending http response: {response:#?}");
    let Some(response_tx) = super::valueof_f2_as_typeof_f1(
        |req: crate::runtime::HttpRequest| req.response_tx,
        || tx.opaque_object_take_data(),
    ) else {
        info!(target: "js::https", "failed to get response tx");
        return;
    };
    if let Err(err) = response_tx.send(crate::runtime::HttpResponseHead {
        status: response.status,
        headers: response.headers.into(),
    }) {
        info!(target: "js::https", "failed to send response: {err:?}");
    }
}
