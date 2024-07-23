use super::*;

use crate::service::OwnedJsValue;
use js::FromJsValue;
use log::{info, trace, warn};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    sync::mpsc::Sender,
};

pub fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("streamBridge", bridge)?;
    ns.define_property_fn("streamOpenWrite", stream_make_writer)?;
    ns.define_property_fn("streamWriteChunk", stream_write_chunk)?;
    ns.define_property_fn("streamOpenRead", stream_make_reader)?;
    ns.define_property_fn("streamClose", stream_close)?;
    Ok(())
}

#[derive(FromJsValue, Debug)]
#[qjs(rename_all = "camelCase")]
struct Args {
    input: js::Value,
    output: js::Value,
}

enum Reader {
    TcpStream(ReadHalf<crate::runtime::TcpStream>),
    DuplexStream(ReadHalf<tokio::io::DuplexStream>),
}

impl Reader {
    fn take(value: &js::Value) -> Option<Self> {
        if let Some(r) = value.opaque_object_take_data() {
            return Some(Reader::TcpStream(r));
        }
        if let Some(r) = value.opaque_object_take_data() {
            return Some(Reader::DuplexStream(r));
        }
        None
    }
    fn dyn_reader(&mut self) -> &mut (dyn tokio::io::AsyncRead + Unpin) {
        match self {
            Reader::TcpStream(reader) => reader,
            Reader::DuplexStream(reader) => reader,
        }
    }
}

enum Writer {
    TcpStream(WriteHalf<crate::runtime::TcpStream>),
    DuplexStream(WriteHalf<tokio::io::DuplexStream>),
}

impl Writer {
    fn take(value: &js::Value) -> Option<Self> {
        if let Some(w) = value.opaque_object_take_data() {
            return Some(Writer::TcpStream(w));
        }
        if let Some(w) = value.opaque_object_take_data() {
            return Some(Writer::DuplexStream(w));
        }
        None
    }
    fn dyn_writer(&mut self) -> &mut (dyn tokio::io::AsyncWrite + Unpin) {
        match self {
            Writer::TcpStream(writer) => writer,
            Writer::DuplexStream(writer) => writer,
        }
    }

    async fn shutdown(&mut self) -> std::io::Result<()> {
        match self {
            Writer::TcpStream(writer) => writer.shutdown().await,
            Writer::DuplexStream(writer) => writer.shutdown().await,
        }
    }
}

#[js::host_call(with_context)]
fn bridge(service: ServiceRef, _this: js::Value, args: Args) -> anyhow::Result<u64> {
    let Some(mut read_half) = Reader::take(&args.input) else {
        anyhow::bail!("failed to get input stream from {:?}", args.input);
    };
    let Some(mut write_half) = Writer::take(&args.output) else {
        anyhow::bail!("failed to get output stream, from {:?}", args.output);
    };
    let id = service.spawn(
        OwnedJsValue::Undefined,
        |_weak_service, _id, _| async move {
            if let Err(err) = tokio::io::copy(read_half.dyn_reader(), write_half.dyn_writer()).await
            {
                warn!(target: "js::stream", "io_bridge: failed to copy data: {err}");
            }
            write_half.shutdown().await.ok();
        },
        (),
    );
    Ok(id)
}

#[derive(FromJsValue)]
struct WriteChunk {
    data: js::Bytes,
    callback: js::Value,
}

#[js::host_call(with_context)]
fn stream_make_writer(
    service: ServiceRef,
    _this: js::Value,
    output_stream: js::Value,
) -> anyhow::Result<js::Value> {
    let Some(mut write_half) = Writer::take(&output_stream) else {
        anyhow::bail!("failed to get output_stream from {output_stream:?}");
    };
    let (tx, rx) = tokio::sync::mpsc::channel::<WriteChunk>(128);
    let _id = service.spawn(
        OwnedJsValue::Null,
        |weak_srv, _id, _| async move {
            let mut rx = rx;
            let write_half = write_half.dyn_writer();
            while let Some(chunk) = rx.recv().await {
                let result = write_half.write_all(chunk.data.as_bytes()).await;
                trace!(target: "js::stream", "{} bytes written, result: {result:?}", chunk.data.len());
                let Some(service) = weak_srv.upgrade() else {
                    warn!(target: "js::stream", "service dropped while writing to stream");
                    break;
                };
                let result = match result {
                    Ok(_) => service.call_function(chunk.callback, (true, js::Value::Null)),
                    Err(err) => service.call_function(chunk.callback, (false, err.to_string())),
                };
                if let Err(err) = result {
                    warn!(target: "js::stream", "failed to report write result: {err:?}");
                }
            }
            trace!(target: "js::stream", "writer done");
            write_half.shutdown().await.ok();
        },
        (),
    );
    Ok(js::Value::new_opaque_object(
        service.context(),
        Some("WriteStream"),
        tx,
    ))
}

#[js::host_call(with_context)]
fn stream_write_chunk(
    service: ServiceRef,
    _this: js::Value,
    writer: js::Value,
    chunk: js::Bytes,
    callback: js::Value,
) -> Result<()> {
    let result = {
        let guard = writer.opaque_object_data::<Sender<WriteChunk>>();
        let Some(tx) = guard.get() else {
            anyhow::bail!("failed to get writer");
        };
        tx.try_send(WriteChunk {
            data: chunk,
            callback: callback.clone(),
        })
    };
    if result.is_err() {
        if let Err(err) = service.call_function(callback, (false, "failed to send chunk")) {
            info!(target: "js::stream", "failed to report write result: {err:?}");
        }
        anyhow::bail!("failed to send chunk: {result:?}");
    }
    Ok(())
}

#[js::host_call]
fn stream_close(writer: js::Value) {
    trace!(target: "js::stream", "closing writer");
    if writer
        .opaque_object_take_data::<Sender<WriteChunk>>()
        .is_none()
    {
        warn!(target: "js::stream", "double drop of writer");
        return;
    };
}

#[js::host_call(with_context)]
fn stream_make_reader(
    service: ServiceRef,
    _this: js::Value,
    input_stream: js::Value,
    callback: OwnedJsValue,
) -> Result<u64> {
    let Some(mut read_half) = Reader::take(&input_stream) else {
        anyhow::bail!("failed to get input_stream from {input_stream:?}");
    };

    let id = service.spawn(
        callback,
        |weak_srv, id, _| async move {
            let mut buf = bytes::BytesMut::with_capacity(super::http_request::STREAM_BUF_SIZE);
            let read_half = read_half.dyn_reader();
            loop {
                buf.clear();
                let result = read_half.read_buf(&mut buf).await;
                let Some(service) = weak_srv.upgrade() else {
                    warn!(target: "js::stream", "service dropped while reading from stream");
                    break;
                };
                let Some(callback) = service.get_resource_value(id) else {
                    warn!(target: "js::stream", "callback dropped while reading from stream");
                    break;
                };
                let mut end = false;
                let result = match result {
                    Ok(0) => {
                        end = true;
                        service.call_function(callback, ("end", js::Value::Null))
                    }
                    Ok(n) => service.call_function(callback, ("data", js::AsBytes(&buf[..n]))),
                    Err(err) => service.call_function(callback, ("error", err.to_string())),
                };
                if let Err(err) = result {
                    warn!(target: "js::stream", "failed to report read result: {err:?}");
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
