use alloc::rc::Weak;
use anyhow::Result;
use log::error;

use crate::service::{Service, ServiceConfig, ServiceRef, ServiceWeakRef};
use crate::traits::ResultExt;

#[cfg(feature = "js-http-listen")]
#[allow(unused_imports)]
pub(crate) use http_listen::try_accept_http_request;
#[cfg(feature = "wapo")]
pub(crate) use query_listen::try_accept_query;

mod debug;
#[cfg(feature = "js-http-listen")]
mod http_listen;
mod http_request;

#[cfg(feature = "js-https-listen")]
mod https_listen;
#[cfg(feature = "mem-stats")]
mod mem_stats;
mod print;
#[cfg(feature = "wapo")]
mod query_listen;
mod timer;
#[cfg(feature = "js-url")]
mod url;
mod wapo_ocalls;

mod derive_secret;

#[cfg(feature = "isolate")]
mod isolate_eval;

mod env;
mod stream;

#[cfg(feature = "js-wasm")]
mod webassambly;

#[cfg(feature = "js-websocket")]
mod websocket;

#[cfg(feature = "js-hash")]
mod hash;

#[cfg(feature = "js-hash")]
mod non_cryptographic_hash;

pub(crate) fn setup_host_functions(ctx: &js::Context, cfg: &ServiceConfig) -> Result<()> {
    let ns = ctx.new_object("Wapo");
    ctx.get_global_object().set_property("Wapo", &ns)?;

    let version = env!("CARGO_PKG_VERSION");
    let version = ctx.new_string(version);
    ns.set_property("version", &version)?;
    set_extensions(&ns, ctx)?;
    print::setup(&ns)?;
    timer::setup(&ns)?;
    http_request::setup(&ns)?;
    debug::setup(&ns)?;
    ns.define_property_fn("close", close_res)?;
    ns.define_property_fn("exit", exit)?;

    #[cfg(feature = "js-url")]
    url::setup(&ns)?;
    #[cfg(feature = "js-hash")]
    {
        hash::setup(&ns)?;
        non_cryptographic_hash::setup(&ns)?;
    }

    #[cfg(feature = "mem-stats")]
    mem_stats::setup(&ns)?;

    stream::setup(&ns)?;

    #[cfg(feature = "js-wasm")]
    webassambly::setup(&ctx.get_global_object())?;

    #[cfg(feature = "js-websocket")]
    websocket::setup(&ns)?;

    if !cfg.is_sandbox {
        env::setup(&ns)?;
        #[cfg(feature = "js-http-listen")]
        http_listen::setup(&ns)?;
        #[cfg(feature = "js-https-listen")]
        https_listen::setup(&ns)?;
        #[cfg(feature = "isolate")]
        isolate_eval::setup(&ns)?;
        #[cfg(feature = "wapo")]
        query_listen::setup(&ns)?;
        wapo_ocalls::setup(&ns)?;
    }

    derive_secret::setup(&ns)?;

    Ok(())
}

fn set_extensions(ns: &js::Value, ctx: &js::Context) -> Result<()> {
    use qjs_extensions as ext;
    let scale = ctx.new_object("SCALE");
    ext::scale2::setup(&scale, ctx)?;
    ext::repr::setup(ns)?;
    ns.set_property("SCALE", &scale)?;
    ns.define_property_fn("hexDecode", ext::hex::decode)?;
    ns.define_property_fn("hexEncode", ext::hex::encode)?;
    ns.define_property_fn("utf8Decode", ext::utf8::decode)?;
    ns.define_property_fn("utf8Encode", ext::utf8::encode)?;
    ns.define_property_fn("base64Decode", ext::base64::decode)?;
    ns.define_property_fn("base64Encode", ext::base64::encode)?;
    #[cfg(feature = "js-crypto")]
    ext::crypto::setup(&ctx.get_global_object())?;
    Ok(())
}

#[no_mangle]
extern "C" fn __pink_getrandom(pbuf: *mut u8, nbytes: u8) {
    let buf = unsafe { core::slice::from_raw_parts_mut(pbuf, nbytes as usize) };
    crate::runtime::getrandom(buf).expect("failed to get random bytes");
}

#[js::host_call(with_context)]
fn close_res(service: ServiceRef, _this: js::Value, res_id: u64) {
    service.remove_resource(res_id);
}

#[js::host_call(with_context)]
fn exit(service: ServiceRef, _this: js::Value) {
    service.close_all();
}

/// This function returns the value of f2 and infer it's type as the return type of f1.
#[allow(dead_code)]
fn valueof_f2_as_typeof_f1<F1, I1, F2, O>(f1: F1, f2: F2) -> Option<O>
where
    F1: FnOnce(I1) -> O,
    F2: FnOnce() -> Option<O>,
{
    let _ = f1;
    f2()
}
