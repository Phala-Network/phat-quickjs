use js::{AsBytes, FromJsValue, ToJsValue};
use log::{info, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc::Sender;

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

#[derive(FromJsValue)]
struct WriteChunk {
    data: AsBytes<Vec<u8>>,
    callback: js::Value,
}

#[derive(ToJsValue, Debug)]
struct Event<'a, Data> {
    name: &'a str,
    data: Data,
}

pub fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("httpListen", http_listen)?;
    ns.define_property_fn("httpSendResponse", http_send_response)?;
    ns.define_property_fn("httpMakeWriter", http_make_writer)?;
    ns.define_property_fn("httpWriteChunk", http_write_chunk)?;
    ns.define_property_fn("httpReceiveBody", http_receive_body)?;
    ns.define_property_fn("httpCloseWriter", http_close_writer)?;
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
        opaque_response_tx: js::Value::new_opaque_object(service.raw_ctx(), request.response_tx),
        opaque_input_stream: js::Value::new_opaque_object(service.raw_ctx(), input_stream),
        opaque_output_stream: js::Value::new_opaque_object(service.raw_ctx(), output_stream),
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

#[js::host_call]
fn http_send_response(tx: js::Value, response: HttpResponseHead) {
    let Some(response_tx) = use_2nd(
        |req: crate::runtime::HttpRequest| req.response_tx,
        || tx.opaque_object_take_data(),
    ) else {
        info!("Failed to get response tx");
        return;
    };
    if let Err(err) = response_tx.send(crate::runtime::HttpResponseHead {
        status: response.status,
        headers: response.headers.into(),
    }) {
        info!("Failed to send response: {err:?}");
    }
}

#[js::host_call(with_context)]
fn http_receive_body(
    service: ServiceRef,
    _this: js::Value,
    input_stream: js::Value,
    callback: OwnedJsValue,
) -> Result<u64> {
    let Some(read_half) = use_2nd(
        |req: crate::runtime::HttpRequest| tokio::io::split(req.io_stream).0,
        || input_stream.opaque_object_take_data(),
    ) else {
        anyhow::bail!("Failed to get input_stream");
    };

    let id = service.spawn(
        callback,
        |weak_srv, id, _| async move {
            let mut reader = BufReader::with_capacity(1024, read_half);
            let mut buf = [0u8; 1024];
            loop {
                let result = reader.read(&mut buf).await;
                let Some(service) = weak_srv.upgrade() else {
                    warn!("Service dropped while reading from stream");
                    break;
                };
                let Some(callback) = service.get_resource_value(id) else {
                    warn!("Callback dropped while reading from stream");
                    break;
                };
                let mut end = false;
                let result = match result {
                    Ok(0) => {
                        end = true;
                        service.call_function(callback, ("end", js::Value::Null))
                    }
                    Ok(n) => service.call_function(callback, ("data", AsBytes(&buf[..n]))),
                    Err(err) => service.call_function(callback, ("error", err.to_string())),
                };
                if let Err(err) = result {
                    warn!("Failed to report read result: {err:?}");
                }
                if end {
                    break;
                }
            }
        },
        (),
    );
    Ok(id)
}

#[js::host_call(with_context)]
fn http_make_writer(
    service: ServiceRef,
    _this: js::Value,
    output_stream: js::Value,
) -> anyhow::Result<js::Value> {
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
                    Ok(_) => service.call_function(chunk.callback, (true, js::Value::Null)),
                    Err(err) => service.call_function(chunk.callback, (false, err.to_string())),
                };
                if let Err(err) = result {
                    warn!("Failed to report write result: {err:?}");
                }
            }
        },
        (),
    );
    Ok(js::Value::new_opaque_object(service.raw_ctx(), tx))
}

#[js::host_call(with_context)]
fn http_write_chunk(
    service: ServiceRef,
    _this: js::Value,
    writer: js::Value,
    chunk: AsBytes<Vec<u8>>,
    callback: js::Value,
) -> Result<()> {
    let Some(tx) = writer.opaque_object_data::<Sender<WriteChunk>>() else {
        anyhow::bail!("Failed to get writer");
    };
    let result = tx.try_send(WriteChunk {
        data: chunk,
        callback: callback.clone(),
    });
    if result.is_err() {
        if let Err(err) = service.call_function(callback, (false, "Failed to send chunk")) {
            info!("Failed to report write result: {err:?}");
        }
        anyhow::bail!("Failed to send chunk");
    }
    Ok(())
}

#[js::host_call]
fn http_close_writer(writer: js::Value) {
    if writer
        .opaque_object_take_data::<Sender<WriteChunk>>()
        .is_none()
    {
        warn!("Double drop of writer");
        return;
    };
}
