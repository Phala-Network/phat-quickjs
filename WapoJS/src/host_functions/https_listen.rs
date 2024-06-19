use anyhow::bail;
use futures::TryStreamExt;
use http_body_util::BodyExt;
use http_request::STREAM_BUF_SIZE;
use hyper1 as hyper;
use js::{FromJsValue, ToJsValue};
use log::{info, trace};
use tokio::io::{duplex, AsyncWriteExt};

use hyper::server::conn::http1::Builder;
use hyper::service::service_fn;
use wapo::hyper_rt::HyperTokioIo;

use super::http_request::Headers;
use super::*;
use crate::runtime::{self, sni_listen, TcpStream, TlsListener};
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
pub struct ServerTlsConfig {
    server_name: js::JsString,
    certificate_chain: js::JsString,
    private_key: js::JsString,
}

#[derive(FromJsValue, Debug)]
#[qjs(rename_all = "camelCase")]
struct HttpResponseHead {
    status: u16,
    headers: Headers,
}

pub fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("httpsListen", https_listen)?;
    ns.define_property_fn("httpsSendResponseHead", https_send_response_head)?;
    Ok(())
}

#[js::host_call(with_context)]
fn https_listen(
    service: ServiceRef,
    _this: js::Value,
    config: ServerTlsConfig,
    callback: OwnedJsValue,
) -> Result<u64> {
    let listener = sni_listen(&config.server_name, &config.certificate_chain, &config.private_key)?;
    let res_id = service.spawn(callback, do_https_listen, listener);
    Ok(res_id)
}

async fn do_https_listen(weak_service: ServiceWeakRef, id: u64, mut listener: TlsListener) {
    while let Ok((stream, _attr)) = listener.accept().await {
        runtime::spawn(serve_connection(weak_service.clone(), id, stream));
    }
    info!(target: "js::https", "https listener terminated");
}

async fn serve_connection(weak_service: ServiceWeakRef, id: u64, connection: TcpStream) {
    let result = Builder::new()
        .serve_connection(
            HyperTokioIo::new(connection),
            service_fn(move |req| {
                let weak_service = weak_service.clone();
                async move {
                let (resposne_tx, response_rx) =
                    tokio::sync::oneshot::channel::<HttpResponseHead>();
                let (up_end, down_end) = duplex(STREAM_BUF_SIZE);
                let (up_end_rx, up_end_tx) = tokio::io::split(up_end);
                let (down_end_rx, mut down_end_tx) = tokio::io::split(down_end);
                {
                    let Some(service) = weak_service.upgrade() else {
                        anyhow::bail!("service dropped");
                    };
                    let opaque_response_tx = js::Value::new_opaque_object(
                        service.context(),
                        Some("HttpResponseTx"),
                        resposne_tx,
                    );
                    let opaque_input_stream = js::Value::new_opaque_object(
                        service.context(),
                        Some("HttpInputBodyStream"),
                        up_end_rx,
                    );
                    let opaque_output_stream = js::Value::new_opaque_object(
                        service.context(),
                        Some("HttpOutputBodyStream"),
                        up_end_tx,
                    );
                    let request = HttpRequest {
                        method: req.method().as_str().to_string(),
                        url: req.uri().to_string(),
                        headers: req
                            .headers()
                            .into_iter()
                            .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap().to_string()))
                            .collect::<Vec<_>>()
                            .into(),
                        opaque_response_tx,
                        opaque_input_stream,
                        opaque_output_stream,
                    };
                    let Some(callback) = service.get_resource_value(id) else {
                        info!(target: "js::https", "dropped https reqest because resource has been dropped");
                        bail!("dropped https reqest because resource has been dropped");
                    };
                    if let Err(err) = service.call_function(callback, (request,)) {
                        error!(target: "js::https", "failed to fire http request event: {err}");
                        bail!("failed to fire http request event: {err}");
                    }
                };
                let mut body = req.into_data_stream();
                // pipe to down_end_tx
                runtime::spawn(async move {
                    loop {
                        let chunk: bytes::Bytes = match body.try_next().await {
                            Ok(Some(chunk)) => chunk,
                            Ok(None) => break,
                            Err(err) => {
                                error!(target: "js::https", "failed to read body: {err}");
                                break;
                            }
                        };
                        if let Err(err) = down_end_tx.write_all(&chunk).await {
                            error!(target: "js::https", "failed to send body: {err}");
                            break;
                        }
                    }
                    down_end_tx.shutdown().await.ok();
                });

                let response_head = match response_rx.await {
                    Ok(head) => head,
                    Err(err) => {
                        error!(target: "js::https", "failed to get response head: {err}");
                        bail!("failed to get response head: {err}");
                    }
                };
                let mut builder = hyper::Response::builder().status(response_head.status);
                for (k, v) in response_head.headers.iter() {
                    builder = builder.header(k, v);
                }

                let stream = tokio_util::io::ReaderStream::new(down_end_rx);
                let stream = stream.map_ok(|chunk| hyper::body::Frame::data(chunk));
                let body = http_body_util::StreamBody::new(stream);
                Ok(builder.body(body)?)
            }}),
        )
        .await;
    if let Err(err) = result {
        error!(target: "js::https", "failed to serve connection: {err}");
    }
}

#[js::host_call]
fn https_send_response_head(tx: js::Value, response: HttpResponseHead) -> Result<()> {
    trace!(target: "js::https", "sending http response: {response:#?}");
    let response_tx =
        match tx.opaque_object_take_data::<tokio::sync::oneshot::Sender<HttpResponseHead>>() {
            Some(response_tx) => response_tx,
            None => {
                info!(target: "js::https", "failed to get response tx");
                bail!("failed to get response tx");
            }
        };
    if response_tx.send(response).is_err() {
        info!(target: "js::https", "failed to send response");
        bail!("failed to send response");
    }
    Ok(())
}
