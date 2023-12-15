use alloc::rc::Weak;
use anyhow::Result;
use log::error;

use crate::service::{Service, ServiceRef, ServiceWeakRef};
use crate::traits::ResultExt;

#[cfg(feature = "js-http-listen")]
pub(crate) use http_listen::try_accept_http_request;

mod debug;
#[cfg(feature = "js-http-listen")]
mod http_listen;
mod http_request;
#[cfg(feature = "mem-stats")]
mod mem_stats;
mod print;
mod timer;
#[cfg(feature = "js-url")]
mod url;

#[cfg(feature = "js-hash")]
mod hash;

pub(crate) fn setup_host_functions(ctx: &js::Context) -> Result<()> {
    let ns = js::Value::new_object(ctx);
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
    #[cfg(feature = "js-http-listen")]
    http_listen::setup(&ns)?;
    #[cfg(feature = "js-hash")]
    hash::setup(&ns)?;
    #[cfg(feature = "mem-stats")]
    mem_stats::setup(&ns)?;

    js::get_global(ctx).set_property("Sidevm", &ns)?;
    setup_process_object(ctx)?;
    Ok(())
}

pub(crate) fn setup_process_object(ctx: &js::Context) -> Result<()> {
    let process = js::Value::new_object(ctx);
    process.define_property_fn("exit", exit)?;
    js::get_global(ctx).set_property("process", &process)?;
    Ok(())
}

fn set_extensions(ns: &js::Value, ctx: &js::Context) -> Result<()> {
    use qjs_extensions as ext;
    let scale = js::Value::new_object(ctx);
    ext::scale2::setup(&scale, ctx)?;
    ext::repr::setup(ns)?;
    ns.set_property("SCALE", &scale)?;
    ns.define_property_fn("hexDecode", ext::hex::decode)?;
    ns.define_property_fn("hexEncode", ext::hex::encode)?;
    ns.define_property_fn("utf8Decode", ext::utf8::decode)?;
    ns.define_property_fn("utf8Encode", ext::utf8::encode)?;
    ns.define_property_fn("base64Decode", ext::base64::decode)?;
    ns.define_property_fn("base64Encode", ext::base64::encode)?;
    Ok(())
}

#[no_mangle]
extern "C" fn __pink_getrandom(pbuf: *mut u8, nbytes: u8) {
    let buf = unsafe { core::slice::from_raw_parts_mut(pbuf, nbytes as usize) };
    crate::runtime::getrandom(buf).expect("Failed to get random bytes");
}

#[js::host_call(with_context)]
fn close_res(service: ServiceRef, _this: js::Value, res_id: u64) {
    service.remove_resource(res_id);
}

#[js::host_call(with_context)]
fn exit(service: ServiceRef, _this: js::Value) {
    service.close_all();
}
