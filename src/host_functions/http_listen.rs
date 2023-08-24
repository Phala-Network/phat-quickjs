use std::collections::BTreeMap;

use log::{info, warn};
use qjs::{host_call, AsBytes, FromJsValue, ToJsValue, Value as JsValue};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Sender;

use super::*;
use crate::service::OwnedJsValue;

#[derive(ToJsValue, Debug)]
#[qjsbind(rename_all = "camelCase")]
pub struct HttpRequest {
    method: String,
    path: String,
    query: String,
    headers: BTreeMap<String, String>,
    opaque_response_tx: JsValue,
    opaque_input_stream: JsValue,
    opaque_output_stream: JsValue,
}

#[derive(FromJsValue, Debug)]
#[qjsbind(rename_all = "camelCase")]
struct HttpResponseHead {
    status: u16,
    headers: BTreeMap<String, String>,
}

#[derive(FromJsValue)]
struct WriteChunk {
    data: AsBytes<Vec<u8>>,
    callback: JsValue,
}

#[derive(ToJsValue, Debug)]
struct Event<'a, Data> {
    name: &'a str,
    data: Data,
}

pub fn setup(ns: &JsValue) -> Result<()> {
    ns.set_property_fn("httpListen", http_listen)?;
    ns.set_property_fn("httpSendResponse", http_send_response)?;
    ns.set_property_fn("httpMakeWriter", http_make_writer)?;
    ns.set_property_fn("httpWriteChunk", http_write_chunk)?;
    ns.set_property_fn("httpCloseWriter", http_close_writer)?;
    Ok(())
}

#[host_call]
fn http_listen(service: ServiceRef, _this: JsValue, callback: OwnedJsValue) {
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
        path: request.head.path.clone(),
        query: request.head.query.clone(),
        headers: request.head.headers.iter().cloned().collect(),
        opaque_response_tx: JsValue::new_opaque_object(service.raw_ctx(), request.response_tx),
        opaque_input_stream: JsValue::new_opaque_object(service.raw_ctx(), input_stream),
        opaque_output_stream: JsValue::new_opaque_object(service.raw_ctx(), output_stream),
    };
    if let Err(err) = service.call_function(listener, (req,)) {
        anyhow::bail!("Failed to fire http request event: {err}");
    }
    Ok(())
}

fn use_2nd<F1, I1, F2, O>(_f1: F1, f2: F2) -> Option<O>
where
    F1: FnOnce(I1) -> O,
    F2: FnOnce() -> Option<O>,
{
    f2()
}

#[host_call]
fn http_send_response(
    _service: ServiceRef,
    _this: JsValue,
    tx: JsValue,
    response: HttpResponseHead,
) {
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

#[host_call]
fn http_make_writer(
    service: ServiceRef,
    _this: JsValue,
    output_stream: JsValue,
) -> anyhow::Result<JsValue> {
    let Some(write_half) = use_2nd(
        |req: crate::runtime::HttpRequest| tokio::io::split(req.io_stream).1,
        || output_stream.opaque_object_take_data(),
    ) else {
        anyhow::bail!("Failed to get output_stream");
    };
    let (tx, rx) = tokio::sync::mpsc::channel::<WriteChunk>(1);
    let _id = service.spawn(
        OwnedJsValue::Null,
        |weak_srv, _id, _| async move {
            let mut rx = rx;
            let mut write_half = write_half;
            while let Some(chunk) = rx.recv().await {
                let result = write_half.write_all(&chunk.data.0).await;
                let Some(service) = weak_srv.upgrade() else {
                    warn!("Service dropped while writing to stream");
                    break;
                };
                let result = match result {
                    Ok(_) => service.call_function(chunk.callback, (true, JsValue::Null)),
                    Err(err) => service.call_function(chunk.callback, (false, err.to_string())),
                };
                if let Err(err) = result {
                    warn!("Failed to report write result: {err:?}");
                }
            }
        },
        (),
    );
    Ok(JsValue::new_opaque_object(service.raw_ctx(), tx))
}

#[host_call]
fn http_write_chunk(
    service: ServiceRef,
    _this: JsValue,
    writer: JsValue,
    chunk: AsBytes<Vec<u8>>,
    callback: JsValue,
) {
    let Some(tx) = writer.opaque_object_data::<Sender<WriteChunk>>() else {
        info!("Failed to get writer");
        return;
    };
    let result = tx.try_send(WriteChunk {
        data: chunk,
        callback: callback.clone(),
    });
    if result.is_err() {
        info!("Failed to send chunk");
        if let Err(err) = service.call_function(callback, (false, "Failed to send chunk")) {
            info!("Failed to report write result: {err:?}");
        }
    }
}

#[host_call]
fn http_close_writer(_service: ServiceRef, _this: JsValue, writer: JsValue) {
    if writer
        .opaque_object_take_data::<Sender<WriteChunk>>()
        .is_none()
    {
        warn!("Double drop of writer");
        return;
    };
}
