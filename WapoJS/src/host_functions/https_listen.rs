use std::fmt::Debug;
use std::str::FromStr;

use anyhow::{bail, Context, Result};
use futures::TryStreamExt;
use http::header::UPGRADE;
use http_body_util::BodyExt;
use http_request::STREAM_BUF_SIZE;
use hyper1 as hyper;
use js::{FromJsValue, ToJsValue};
use log::{info, trace};
use tokio::io::{duplex, AsyncWriteExt, DuplexStream, ReadHalf, WriteHalf};

use hyper::body::{Body, Frame, SizeHint};
use hyper::server::conn::http1::Builder;
use hyper::service::service_fn;
use tokio::sync::oneshot;
use wapo::hyper_rt::HyperTokioIo;

use super::http_request::Headers;
use super::*;
use crate::runtime::{self, sni_listen, TcpStream, TlsListener};
use crate::service::OwnedJsValue;

#[pin_project::pin_project(project = ResponseBodyProj)]
enum ResponseBody<T> {
    Stream(#[pin] T),
    Empty,
}

impl<T: Body> Body for ResponseBody<T>
where
    T::Data: Debug,
    T::Error: Debug,
{
    type Data = T::Data;
    type Error = T::Error;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let result = match self.project() {
            ResponseBodyProj::Stream(body) => body.poll_frame(cx),
            ResponseBodyProj::Empty => std::task::Poll::Ready(None),
        };
        trace!(target: "js::https", "response body poll_frame: {result:?}");
        result
    }

    fn is_end_stream(&self) -> bool {
        match self {
            ResponseBody::Stream(body) => body.is_end_stream(),
            ResponseBody::Empty => true,
        }
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            ResponseBody::Stream(body) => body.size_hint(),
            ResponseBody::Empty => SizeHint::with_exact(0),
        }
    }
}

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

impl FromStr for HttpResponseHead {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let todo = "use standard parser";
        let mut lines = s.lines();
        let status = lines
            .next()
            .context("failed to parse status line")?
            .split_whitespace()
            .nth(1)
            .context("no status code found")?
            .parse()
            .context("failed to parse status code")?;
        let headers = lines
            .map(|line| {
                let mut parts = line.splitn(2, ": ");
                let key = parts.next().context("failed to parse header key")?;
                let value = parts.next().context("failed to parse header value")?;
                Ok((key.to_string(), value.to_string()))
            })
            .collect::<Result<Headers>>()?;
        Ok(HttpResponseHead { status, headers })
    }
}

pub fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("httpsListen", https_listen)?;
    ns.define_property_fn("httpsSendResponseHead", https_send_response_head)?;
    ns.define_property_fn("httpsSendResponseHeadRaw", https_send_response_head_raw)?;
    Ok(())
}

#[js::host_call(with_context)]
fn https_listen(
    service: ServiceRef,
    _this: js::Value,
    config: ServerTlsConfig,
    callback: OwnedJsValue,
) -> Result<u64> {
    let listener = sni_listen(
        &config.server_name,
        &config.certificate_chain,
        &config.private_key,
    )?;
    let res_id = service.spawn(callback, do_https_listen, listener);
    Ok(res_id)
}

async fn do_https_listen(weak_service: ServiceWeakRef, id: u64, mut listener: TlsListener) {
    while let Ok((stream, addr)) = listener.accept().await {
        trace!(target: "js::https", "https connection accepted: {addr:?}");
        runtime::spawn(serve_connection(weak_service.clone(), id, stream));
    }
    info!(target: "js::https", "https listener terminated");
}

async fn wait_for_response(
    response_rx: oneshot::Receiver<HttpResponseHead>,
) -> Result<http::response::Builder> {
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
    Ok(builder)
}

async fn serve_connection(weak_service: ServiceWeakRef, id: u64, connection: TcpStream) {
    let result = Builder::new()
        .serve_connection(
            HyperTokioIo::new(connection),
            service_fn(move |req| {
                let weak_service = weak_service.clone();
                async move {
                    if req.method() == hyper::Method::OPTIONS {
                        let res = hyper::Response::builder()
                            .status(204)
                            .header("Allow", "OPTIONS, GET, POST")
                            .body(ResponseBody::Empty)?;
                        return Ok(res);
                    }
                    let (resposne_tx, response_rx) =
                        oneshot::channel::<HttpResponseHead>();
                    let (up_end, down_end) = duplex(STREAM_BUF_SIZE);
                    let (from_c_rx, to_c_tx) = tokio::io::split(up_end);
                    let (from_s_rx, mut to_s_tx) = tokio::io::split(down_end);
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
                            from_c_rx,
                        );
                        let opaque_output_stream = js::Value::new_opaque_object(
                            service.context(),
                            Some("HttpOutputBodyStream"),
                            to_c_tx,
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
                        trace!(target: "js::https", "http request: {request:#?}");
                        let Some(callback) = service.get_resource_value(id) else {
                            info!(target: "js::https", "dropped https reqest because resource has been dropped");
                            bail!("dropped https reqest because resource has been dropped");
                        };
                        if let Err(err) = service.call_function(callback, (request,)) {
                            error!(target: "js::https", "failed to fire http request event: {err}");
                            bail!("failed to fire http request event: {err}");
                        }
                    };
                    match req.headers().contains_key(UPGRADE) {
                        false => {
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
                                    if let Err(err) = to_s_tx.write_all(&chunk).await {
                                        error!(target: "js::https", "failed to send body: {err}");
                                        break;
                                    }
                                }
                                to_s_tx.shutdown().await.ok();
                            });
                            let builder = wait_for_response(response_rx).await?;
                            let stream = tokio_util::io::ReaderStream::new(from_s_rx);
                            let stream = stream.map_ok(|chunk| hyper::body::Frame::data(chunk));
                            let body = ResponseBody::Stream(http_body_util::StreamBody::new(stream));
                            Ok(builder.body(body)?)
                        }
                        true => {
                            let builder = wait_for_response(response_rx).await?;
                            let response = builder.body(ResponseBody::Empty).context("failed to build response")?;
                            if response.headers().contains_key(UPGRADE) {
                                runtime::spawn(async move {
                                    let mut req = req;
                                    match hyper::upgrade::on(&mut req).await {
                                        Ok(upgraded) => {
                                            if let Err(e) = server_upgraded_io(upgraded, to_s_tx, from_s_rx).await {
                                                error!(target: "js::https", "server io error: {}", e)
                                            };
                                        }
                                        Err(e) => error!(target: "js::https", "upgrade error: {}", e),
                                    }
                                });
                            }
                            Ok(response)
                        }
                    }
                }}),
        )
        .with_upgrades()
        .await;
    if let Err(err) = result {
        error!(target: "js::https", "failed to serve connection: {err}");
    }
}

async fn server_upgraded_io(
    upgraded: hyper::upgrade::Upgraded,
    mut to_s_tx: WriteHalf<DuplexStream>,
    mut from_s_rx: ReadHalf<DuplexStream>,
) -> Result<()> {
    trace!(target: "js::https", "upgraded io");
    let upgraded = HyperTokioIo::new(upgraded);
    let (mut from_c_rx, mut to_c_tx) = tokio::io::split(upgraded);
    let c2s = tokio::io::copy(&mut from_c_rx, &mut to_s_tx);
    let s2c = tokio::io::copy(&mut from_s_rx, &mut to_c_tx);
    tokio::select! {
        _ = c2s => {
            trace!(target: "js::https", "upgraded c2s closed");
        }
        _ = s2c => {
            trace!(target: "js::https", "upgraded s2c closed");
        }
    }
    to_c_tx.shutdown().await.ok();
    to_s_tx.shutdown().await.ok();
    Ok(())
}

#[js::host_call]
fn https_send_response_head(tx: js::Value, response: HttpResponseHead) -> Result<()> {
    do_https_send_response_head(tx, response)
}

#[js::host_call]
fn https_send_response_head_raw(tx: js::Value, head: js::JsString) -> Result<()> {
    do_https_send_response_head(tx, head.parse().context("failed to parse response head")?)
}

fn do_https_send_response_head(tx: js::Value, response: HttpResponseHead) -> Result<()> {
    trace!(target: "js::https", "sending http response: {response:#?}");
    let response_tx = match tx.opaque_object_take_data::<oneshot::Sender<HttpResponseHead>>() {
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
