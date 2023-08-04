use alloc::rc::Weak;
use log::{debug, error};
use qjs_sys::{
    c,
    convert::{serialize_value, DecodeFromJSValue, JsValue},
};

use crate::service::{js_context_get_service, Resource, Service, ServiceRef, ServiceWeakRef};

mod timer;

trait IntoJsValue {
    fn into_js_value(self, ctx: *mut c::JSContext) -> c::JSValue;
}

impl<T: AsRef<str>> IntoJsValue for Result<c::JSValue, T> {
    fn into_js_value(self, ctx: *mut c::JSContext) -> c::JSValue {
        match self {
            Ok(v) => v,
            Err(err) => {
                qjs_sys::throw_type_error(ctx, err.as_ref());
                c::JS_EXCEPTION
            }
        }
    }
}

#[no_mangle]
fn __pink_host_call(id: u32, ctx: *mut c::JSContext, args: &[c::JSValueConst]) -> c::JSValue {
    let Some(service) = js_context_get_service(ctx) else {
        qjs_sys::throw_type_error(ctx, "Host call without service");
        return c::JS_EXCEPTION;
    };
    let Some(service) = service.upgrade() else {
        qjs_sys::throw_type_error(ctx, "Host call while the service is dropped");
        return c::JS_EXCEPTION;
    };
    match id {
        1001 => timer::set_timeout(service, ctx, args).into_js_value(ctx),
        1002 => timer::clear_timeout(service, ctx, args).into_js_value(ctx),
        1003 => timer::set_interval(service, ctx, args).into_js_value(ctx),
        1004 => timer::clear_timeout(service, ctx, args).into_js_value(ctx), // Reusing clear_timeout
        _ => {
            error!("JS: host call with unknown id: {id}");
            qjs_sys::throw_type_error(ctx, &alloc::format!("Invalid host call id: {id}"));
            c::JS_EXCEPTION
        }
    }
}

#[no_mangle]
extern "C" fn __pink_getrandom(pbuf: *mut u8, nbytes: u8) {
    let buf = unsafe { core::slice::from_raw_parts_mut(pbuf, nbytes as usize) };
    sidevm::ocall::getrandom(buf).expect("Failed to get random bytes");
}
