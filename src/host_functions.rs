use alloc::rc::Weak;
use log::{debug, error};
use qjs_sys::{
    c,
    convert::{serialize_value, DecodeFromJSValue, JsValue},
};

use crate::service::{js_context_get_service, Resource, Service, ServiceRef, ServiceWeakRef};
use anyhow::{anyhow, Error, Result};

mod timer;
mod http_request;

#[no_mangle]
fn __pink_host_call(id: u32, ctx: *mut c::JSContext, args: &[c::JSValueConst]) -> c::JSValue {
    let result = do_host_call(id, ctx, args)
        .and_then(|value| serialize_value(ctx, value).map_err(Error::msg));
    match result {
        Ok(value) => value,
        Err(err) => {
            let err = format!("{err}");
            qjs_sys::throw_type_error(ctx, &err);
            c::JS_EXCEPTION
        }
    }
}

fn do_host_call(id: u32, ctx: *mut c::JSContext, args: &[c::JSValueConst]) -> Result<JsValue> {
    let service = js_context_get_service(ctx)
        .ok_or(anyhow!("Host call without a service attached"))?
        .upgrade()
        .ok_or(anyhow!("Host call while the service is dropped"))?;
    let value = match id {
        1001 => timer::set_timeout(service, ctx, args)?,
        1002 => drop_resource(service, ctx, args)?,
        1003 => timer::set_interval(service, ctx, args)?,
        1004 => drop_resource(service, ctx, args)?,
        _ => anyhow::bail!("Invalid host call id: {id}"),
    };
    Ok(value)
}

#[no_mangle]
extern "C" fn __pink_getrandom(pbuf: *mut u8, nbytes: u8) {
    let buf = unsafe { core::slice::from_raw_parts_mut(pbuf, nbytes as usize) };
    sidevm::ocall::getrandom(buf).expect("Failed to get random bytes");
}

fn drop_resource(
    service: ServiceRef,
    ctx: *mut c::JSContext,
    args: &[c::JSValueConst],
) -> Result<JsValue> {
    let id: u64 = match args.get(0) {
        Some(id) => DecodeFromJSValue::decode(ctx, *id).map_err(Error::msg)?,
        None => anyhow::bail!("Invoking clearTimeout without id"),
    };
    service.remove_resource(id);
    Ok(JsValue::Null)
}
