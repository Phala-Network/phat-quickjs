use anyhow::{anyhow, Context};
use log::{info, warn};
use std::{collections::BTreeMap, time::Duration};
use tokio::io::{AsyncReadExt, DuplexStream, ReadHalf, WriteHalf};

use crate::{runtime::time::sleep, service::OwnedJsValue};
use js::{Error as ValueError, FromJsValue, ToJsValue};

use super::*;

#[derive(Debug, Default)]
pub struct Headers {
    pairs: Vec<(String, String)>,
}

impl FromJsValue for Headers {
    fn from_js_value(value: js::Value) -> Result<Self, ValueError> {
        Ok(if value.is_array() {
            Vec::<(String, String)>::from_js_value(value)?.into()
        } else {
            BTreeMap::<String, String>::from_js_value(value)?.into()
        })
    }
}

impl ToJsValue for Headers {
    fn to_js_value(&self, ctx: &js::Context) -> Result<js::Value, ValueError> {
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
    #[qjsbind(default)]
    body: js::BytesOrString,
    #[qjsbind(default)]
    stream_body: bool,
    #[qjsbind(default = "default_timeout")]
    timeout_ms: u64,
}

#[derive(ToJsValue, Debug)]
#[qjsbind(rename_all = "camelCase")]
pub struct HttpRequestReceipt {
    cancel_token: u64,
    opaque_body_stream: Option<js::Value>,
}

#[derive(ToJsValue, Debug)]
#[qjsbind(rename_all = "camelCase")]
struct HttpResponseHead {
    status: u16,
    status_text: String,
    version: String,
    headers: Headers,
    opaque_body_stream: js::Value,
}

#[derive(ToJsValue, Debug)]
struct Event<'a, Data> {
    name: &'a str,
    data: Data,
}

pub fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("httpRequest", http_request)?;
    Ok(())
}

pub(crate) const STREAM_BUF_SIZE: usize = 8192;
struct Pipes {
    duplex_up: DuplexStream,
    duplex_down_rx: ReadHalf<DuplexStream>,
}

impl Pipes {
    fn create() -> (WriteHalf<DuplexStream>, Self) {
        let (duplex_up, duplex_down) = tokio::io::duplex(STREAM_BUF_SIZE);
        let (duplex_down_rx, duplex_down_tx) = tokio::io::split(duplex_down);
        (
            duplex_down_tx,
            Self {
                duplex_up,
                duplex_down_rx,
            },
        )
    }
}

#[js::host_call(with_context)]
fn http_request(
    service: ServiceRef,
    _this: js::Value,
    req: HttpRequest,
    callback: OwnedJsValue,
) -> Result<HttpRequestReceipt> {
    let (duplex_down_tx, pipes) = Pipes::create();
    let opaque_body_stream = if req.stream_body {
        Some(js::Value::new_opaque_object(
            service.context(),
            duplex_down_tx,
        ))
    } else {
        None
    };
    let cancel_token = service.spawn(callback, do_http_request, (req, pipes));
    Ok(HttpRequestReceipt {
        cancel_token,
        opaque_body_stream,
    })
}

fn default_method() -> String {
    "GET".into()
}

fn default_timeout() -> u64 {
    30_000
}

async fn do_http_request(
    weak_service: ServiceWeakRef,
    id: u64,
    (req, pipes): (HttpRequest, Pipes),
) {
    let url = req.url.clone();
    let result = tokio::select! {
        _ = sleep(Duration::from_millis(req.timeout_ms)) => {
            Err(anyhow!("timed out"))
        }
        result = do_http_request_inner(weak_service.clone(), id, req, pipes) => result,
    };
    if let Err(err) = result {
        warn!("failed to request `{url}`: {err:?}");
        invoke_callback(
            &weak_service,
            id,
            "error",
            &format!("failed to request `{url}`: {err:?}"),
        );
    }
}

async fn do_http_request_inner(
    weak_service: ServiceWeakRef,
    id: u64,
    req: HttpRequest,
    pipes: Pipes,
) -> Result<()> {
    use crate::runtime::{http_connector, HyperExecutor};
    use core::pin::pin;
    use hyper::{body::HttpBody, Body};
    use tokio::io::AsyncWriteExt;
    let connector = http_connector();
    let client = hyper::Client::builder()
        .executor(HyperExecutor)
        .build::<_, Body>(connector);
    let uri: hyper::Uri = req
        .url
        .parse()
        .with_context(|| format!("failed to parse url: {}", req.url))?;
    let mut builder = hyper::Request::builder()
        .method(req.method.to_uppercase().as_str())
        .uri(&uri);
    for (k, v) in req.headers.pairs.iter() {
        builder = builder.header(k.as_str(), v.as_str());
    }
    let headers_map = builder
        .headers_mut()
        .ok_or_else(|| anyhow!("failed to build request"))?;
    // Append Host, Content-Length and User-Agent if not present
    if !headers_map.contains_key("Host") {
        headers_map.insert("Host", uri.host().unwrap_or_default().parse()?);
    }
    if !headers_map.contains_key("User-Agent") {
        headers_map.insert("User-Agent", "WapoJS/0.1.0".parse()?);
    }
    let (body_tx, body);
    if req.stream_body {
        let (tx, b) = Body::channel();
        body_tx = Some(tx);
        body = b;
    } else {
        let body_bytes = req.body.as_bytes().to_vec();
        if !headers_map.contains_key("Content-Length") {
            headers_map.insert("Content-Length", body_bytes.len().to_string().parse()?);
        }
        body = body_bytes.into();
        body_tx = None;
    }
    let request = builder.body(body).context("failed to build request")?;
    let (mut duplex_up_rx, mut duplex_up_tx) = tokio::io::split(pipes.duplex_up);
    if let Some(mut body_tx) = body_tx {
        crate::runtime::spawn(async move {
            loop {
                let mut buf = bytes::BytesMut::with_capacity(STREAM_BUF_SIZE);
                let chunk = match duplex_up_rx.read_buf(&mut buf).await {
                    Ok(n) if n == 0 => break,
                    Ok(_n) => buf.into(),
                    Err(err) => {
                        warn!("failed to read request body from pipe: {err}");
                        break;
                    }
                };
                if body_tx.send_data(chunk).await.is_err() {
                    warn!("failed to write request body to pipe");
                    break;
                }
            }
        });
    }
    {
        let response = client.request(request).await?;
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
            crate::runtime::spawn(async move {
                let mut response = pin!(response);
                // TODO: add timeout?
                while let Some(chunk) = response.data().await {
                    let Ok(chunk) = chunk else {
                        warn!("failed to read response body");
                        break;
                    };
                    if duplex_up_tx.write_all(&chunk).await.is_err() {
                        warn!("failed to write response body to pipe");
                        break;
                    }
                }
                duplex_up_tx.shutdown().await.ok();
            });
            let service = weak_service
                .clone()
                .upgrade()
                .ok_or_else(|| anyhow!("service dropped while reading response body"))?;
            let opaque_body_stream =
                js::Value::new_opaque_object(service.context(), pipes.duplex_down_rx);
            HttpResponseHead {
                status,
                status_text,
                version,
                headers,
                opaque_body_stream,
            }
        };
        invoke_callback(&weak_service, id, "head", &head);
    }
    Ok(())
}

fn invoke_callback(weak_service: &Weak<Service>, id: u64, name: &str, data: &dyn ToJsValue) {
    let Some(service) = weak_service.upgrade() else {
        info!("http_request {id} exited because the service has been dropped");
        return;
    };
    let Some(callback) = service.get_resource_value(id) else {
        info!("http_request {id} exited because the resource has been dropped");
        return;
    };
    if let Err(err) = service.call_function(callback, (name, data)) {
        error!("[{id}] failed to report http_request event {name}: {err:?}");
    }
}
