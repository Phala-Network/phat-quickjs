use js::{AsBytes, ToJsValue};
use log::{debug, info};

use super::*;
use crate::service::OwnedJsValue;

#[derive(ToJsValue, Debug)]
#[qjs(rename_all = "camelCase")]
pub struct Query {
    origin: Option<AsBytes<[u8; 32]>>,
    path: String,
    payload: AsBytes<Vec<u8>>,
    reply_tx: js::Value,
}

pub fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("queryListen", query_listen)?;
    ns.define_property_fn("queryReply", query_reply)?;
    Ok(())
}

#[js::host_call(with_context)]
fn query_listen(service: ServiceRef, _this: js::Value, callback: OwnedJsValue) {
    service.set_query_listener(callback)
}

pub(crate) fn try_accept_query(service: ServiceRef, request: wapo::channel::Query) -> Result<()> {
    let Some(Ok(listener)) = service.query_listener().map(TryInto::try_into) else {
        debug!(target: "js::query", "no query listener, ignoring request");
        return Ok(());
    };
    debug!(target: "js::query", "accepting query request: {:#?}", request.path);
    let req = Query {
        reply_tx: js::Value::new_opaque_object(
            service.context(),
            Some("QueryReplyTx"),
            request.reply_tx,
        ),
        origin: request.origin.map(AsBytes),
        path: request.path,
        payload: AsBytes(request.payload),
    };
    if let Err(err) = service.call_function(listener, (req,)) {
        anyhow::bail!("failed to fire http request event: {err}");
    }
    Ok(())
}

#[js::host_call]
fn query_reply(tx: js::Value, data: js::BytesOrString) {
    let Some(reply_tx) = super::valueof_f2_as_typeof_f1(
        |req: wapo::channel::Query| req.reply_tx,
        || tx.opaque_object_take_data(),
    ) else {
        info!(target: "js::query", "failed to get response tx");
        return;
    };
    if let Err(err) = reply_tx.send(data.as_bytes()) {
        info!(target: "js::query", "failed to send response: {err:?}");
    }
}
