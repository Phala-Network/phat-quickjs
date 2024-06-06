use anyhow::{bail, Context};
use async_tungstenite::tungstenite::{protocol::WebSocketConfig, Message};
use futures::{SinkExt as _, StreamExt};
use log::{debug, info, trace, warn};
use std::collections::BTreeMap;
use tokio_util::compat::TokioAsyncReadCompatExt as _;

use crate::{runtime, service::OwnedJsValue};
use js::{Error as ValueError, FromJsValue, ToJsValue};

use super::*;

type WsSink = tokio::sync::mpsc::Sender<Message>;

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
#[derive(FromJsValue, ToJsValue, Debug)]
#[qjs(rename_all = "camelCase")]
pub struct WsMessage {
    pub kind: String,
    #[qjs(default)]
    pub data: js::BytesOrString,
}

impl From<Message> for WsMessage {
    fn from(value: Message) -> Self {
        match value {
            Message::Text(text) => Self {
                kind: "text".to_string(),
                data: js::BytesOrString::String(text),
            },
            Message::Binary(bin) => Self {
                kind: "binary".to_string(),
                data: js::BytesOrString::Bytes(bin),
            },
            Message::Ping(data) => Self {
                kind: "ping".to_string(),
                data: js::BytesOrString::Bytes(data),
            },
            Message::Pong(data) => Self {
                kind: "pong".to_string(),
                data: js::BytesOrString::Bytes(data),
            },
            Message::Close(_) => Self {
                kind: "close".to_string(),
                data: js::BytesOrString::String("".to_string()),
            },
            Message::Frame(data) => Self {
                kind: "frame".to_string(),
                data: js::BytesOrString::Bytes(data.into_data()),
            },
        }
    }
}

impl TryFrom<WsMessage> for Message {
    type Error = js::Error;

    fn try_from(value: WsMessage) -> Result<Self, Self::Error> {
        match value.kind.as_str() {
            "text" => Ok(Message::Text(
                value.data.as_str().unwrap_or_default().to_string(),
            )),
            "binary" => Ok(Message::Binary(value.data.as_bytes().to_vec())),
            "ping" => Ok(Message::Ping(value.data.as_bytes().to_vec())),
            "pong" => Ok(Message::Pong(value.data.as_bytes().to_vec())),
            "close" => Ok(Message::Close(None)),
            "frame" => Ok(Message::Binary(value.data.as_bytes().to_vec())),
            _ => Err(js::Error::msg("invalid message kind")),
        }
    }
}

#[derive(FromJsValue, Debug)]
#[qjs(rename_all = "camelCase")]
pub struct WsConfig {
    pub write_buffer_size: Option<usize>,
    pub max_write_buffer_size: Option<usize>,
    pub max_message_size: Option<usize>,
    pub max_frame_size: Option<usize>,
    pub accept_unmasked_frames: Option<bool>,
}

impl From<WsConfig> for WebSocketConfig {
    fn from(config: WsConfig) -> Self {
        let mut ws_config = WebSocketConfig::default();
        if let Some(size) = config.write_buffer_size {
            ws_config.write_buffer_size = size;
        }
        if let Some(size) = config.max_write_buffer_size {
            ws_config.max_write_buffer_size = size;
        }
        if let Some(size) = config.max_message_size {
            ws_config.max_message_size = Some(size);
        }
        if let Some(size) = config.max_frame_size {
            ws_config.max_frame_size = Some(size);
        }
        if let Some(accept_unmasked_frames) = config.accept_unmasked_frames {
            ws_config.accept_unmasked_frames = accept_unmasked_frames;
        }
        ws_config
    }
}

#[derive(FromJsValue, Debug)]
#[qjs(rename_all = "camelCase")]
pub struct OpenOptions {
    url: String,
    #[qjs(default)]
    headers: Headers,
    config: Option<WsConfig>,
}

pub fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("wsOpen", ws_open)?;
    ns.define_property_fn("wsSend", ws_send)?;
    ns.define_property_fn("wsClose", ws_close)?;
    Ok(())
}

#[js::host_call(with_context)]
fn ws_open(
    service: ServiceRef,
    _this: js::Value,
    options: OpenOptions,
    callback: OwnedJsValue,
) -> Result<u64> {
    debug!(target: "js::ws", "opening ws: {}", options.url);
    trace!(target: "js::ws", "ws options: {:?}", options);
    let cancel_token = service.spawn(callback, do_ws_open, options);
    trace!(target: "js::ws", "opened ws {cancel_token}");
    Ok(cancel_token)
}

async fn do_ws_open(weak_service: ServiceWeakRef, id: u64, options: OpenOptions) {
    let url = options.url.clone();
    let result = do_ws_open_inner(weak_service.clone(), id, options).await;
    if let Err(err) = result {
        warn!(target: "js::ws", "failed to open ws `{url}`: {err:?}");
        invoke_callback(
            &weak_service,
            id,
            "error",
            &format!("failed to request `{url}`: {err:?}"),
        );
    }
}

async fn do_ws_open_inner(
    weak_service: ServiceWeakRef,
    id: u64,
    options: OpenOptions,
) -> Result<()> {
    let request = {
        let mut builder = http::Request::builder().method("GET").uri(&options.url);
        for (name, value) in options.headers.pairs {
            builder = builder.header(name, value);
        }
        builder.body(()).context("failed to build request")?
    };
    let ws_config = options.config.map(Into::into);
    let url: http::Uri = options.url.parse().context("invalid url")?;
    let use_tls = url.scheme_str() == Some("wss");
    let host = url.host().context("missing host")?;
    let port = url.port_u16().unwrap_or(if use_tls { 443 } else { 80 });
    let stream = runtime::TcpStream::connect(host, port, use_tls)
        .await
        .context("failed to connect to ws server")?;
    trace!(target: "js::ws", "tcp connected to ws server: {url}");
    let (ws_stream, _response) =
        async_tungstenite::client_async_with_config(request, stream.compat(), ws_config)
            .await
            .context("failed to open ws connection")?;
    trace!(target: "js::ws", "ws {id} handshake down");
    let (mut tx, mut rx) = ws_stream.split();
    {
        let service = weak_service.upgrade().context("service dropped")?;
        let (ch_tx, mut ch_rx) = tokio::sync::mpsc::channel(10);
        runtime::spawn(async move {
            while let Some(msg) = ch_rx.recv().await {
                trace!(target: "js::ws", "sending ws message: {:?}", msg);
                if let Err(err) = tx.send(msg).await {
                    warn!(target: "js::ws", "failed to send ws message: {err:?}");
                    break;
                }
            }
            trace!(target: "js::ws", "ws {id} closed");
            tx.send(Message::Close(None)).await.ok();
        });
        let js_tx =
            js::Value::new_opaque_object::<WsSink>(service.context(), Some("WsSink"), ch_tx);
        trace!(target: "js::ws", "ws {id} opened");
        invoke_callback(&weak_service, id, "open", &js_tx);
    }
    loop {
        let Some(msg) = rx.next().await else {
            info!(target: "js::ws", "client closed ws {id}");
            break;
        };
        match msg {
            Ok(msg) => {
                trace!(target: "js::ws", "ws {id} received message: {msg:?}");
                let msg: WsMessage = msg.into();
                invoke_callback(&weak_service, id, "message", &msg);
            }
            Err(err) => {
                warn!(target: "js::ws", "ws {id} failed to receive message: {err:?}");
                break;
            }
        }
    }
    Ok(())
}

fn invoke_callback(weak_service: &Weak<Service>, id: u64, name: &str, data: &dyn ToJsValue) {
    let Some(service) = weak_service.upgrade() else {
        info!(target: "js::ws", "ws {id} exited because the service has been dropped");
        return;
    };
    let Some(callback) = service.get_resource_value(id) else {
        info!(target: "js::ws", "ws {id} exited because the resource has been dropped");
        return;
    };
    if let Err(err) = service.call_function(callback, (name, data)) {
        error!(target: "js::ws", "[{id}] failed to report ws event {name}: {err:?}");
    }
}

#[js::host_call]
fn ws_send(tx: js::Value, msg: WsMessage) -> Result<()> {
    trace!(target: "js::ws", "sending ws message: {msg:?}");
    let mut tx = tx.opaque_object_data_mut::<WsSink>();
    let msg = msg.try_into().context("invalid message")?;
    tx.get_mut()
        .context("closed")?
        .try_send(msg)
        .context("failed to send message")?;
    Ok(())
}

#[js::host_call]
fn ws_close(tx: js::Value) -> Result<()> {
    trace!(target: "js::ws", "closing ws");
    let Some(tx) = tx.opaque_object_take_data::<WsSink>() else {
        bail!("already closed");
    };
    drop(tx);
    Ok(())
}
