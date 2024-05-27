use super::*;

use crate::service::OwnedJsValue;
use js::FromJsValue;
use log::warn;
use tokio::io::{AsyncWriteExt, ReadHalf, WriteHalf};

pub fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("ioBridge", io_bridge)?;
    Ok(())
}

#[derive(FromJsValue, Debug)]
#[qjsbind(rename_all = "camelCase")]
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
fn io_bridge(service: ServiceRef, _this: js::Value, args: Args) -> anyhow::Result<u64> {
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
                warn!("io_bridge: failed to copy data: {err}");
            }
            write_half.shutdown().await.ok();
        },
        (),
    );
    Ok(id)
}
