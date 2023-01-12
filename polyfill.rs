use alloc::string::String;
use core::ffi::{c_int, c_uchar};
use pink_extension::{error, info};
use qjs_sys::c;
use qjs_sys::convert::{js_val_from_bytes, js_val_into_bytes, js_val_into_u128};

use crate::contract_call;

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
extern "C" fn __pink_fd_write(fd: c_int, buf: *const c_uchar, len: usize) -> usize {
    // TODO: a more robust implementation.
    let bin = unsafe { core::slice::from_raw_parts(buf, len) };
    let message = core::str::from_utf8(bin)
        .unwrap_or("<Invalid UTF-8 string>")
        .trim_end();
    if message.is_empty() {
        return len;
    }
    match fd {
        1 => info!("JS: {}", message),
        2 => error!("JS: {}", message),
        _ => {}
    }
    len
}

#[no_mangle]
extern "C" fn __pink_clock_time_get(_id: u32, _precision: u64, retptr0: *mut u64) -> u16 {
    let t = pink_extension::ext().untrusted_millis_since_unix_epoch() * 1_000_000;
    unsafe {
        *retptr0 = t;
    }
    0
}

macro_rules! cs {
    ($s: literal) => {
        cstr::cstr!($s).as_ptr()
    };
}

#[no_mangle]
fn __pink_host_call(id: u32, ctx: *mut c::JSContext, args: &[c::JSValueConst]) -> c::JSValue {
    match id {
        0 => host_invoke_contract(ctx, args).into_js_value(ctx),
        1 => host_invoke_contract_delegate(ctx, args).into_js_value(ctx),
        _ => {
            error!("JS: host call with unknown id: {id}");
            qjs_sys::throw_type_error(ctx, &alloc::format!("Invalid host call id: {id}"));
            c::JS_EXCEPTION
        }
    }
}

fn host_invoke_contract(
    ctx: *mut c::JSContext,
    args: &[c::JSValueConst],
) -> Result<c::JSValue, String> {
    let Some(config) = args.get(0) else {
        return Err("Invoking contract without arguments".into());
    };

    let callee = unsafe { c::JS_GetPropertyStr(ctx, *config, cs!("callee")) };
    let callee: [u8; 32] = js_val_into_bytes(ctx, callee)?
        .try_into()
        .or(Err("invalid callee"))?;
    let gas_limit = unsafe { c::JS_GetPropertyStr(ctx, *config, cs!("gasLimit")) };
    let gas_limit = js_val_into_u128(ctx, gas_limit)? as u64;
    let transferred_value = unsafe { c::JS_GetPropertyStr(ctx, *config, cs!("value")) };
    let transferred_value = js_val_into_u128(ctx, transferred_value)?;
    let selector = unsafe { c::JS_GetPropertyStr(ctx, *config, cs!("selector")) };
    let selector = js_val_into_u128(ctx, selector)? as u32;
    let input = unsafe { c::JS_GetPropertyStr(ctx, *config, cs!("input")) };
    let input = js_val_into_bytes(ctx, input)?;

    let output = contract_call::invoke_contract(
        callee.into(),
        gas_limit,
        transferred_value,
        selector,
        &input,
    )
    .map_err(|err| alloc::format!("{:?}", err))?;
    Ok(js_val_from_bytes(ctx, &output))
}

fn host_invoke_contract_delegate(
    ctx: *mut c::JSContext,
    args: &[c::JSValueConst],
) -> Result<c::JSValue, String> {
    let Some(config) = args.get(0) else {
        return Err("Invoking contract delegate without arguments".into());
    };

    let delegate = unsafe { c::JS_GetPropertyStr(ctx, *config, cs!("codeHash")) };
    let delegate: [u8; 32] = js_val_into_bytes(ctx, delegate)?
        .try_into()
        .or(Err("invalid delegate"))?;
    let selector = unsafe { c::JS_GetPropertyStr(ctx, *config, cs!("selector")) };
    let selector = js_val_into_u128(ctx, selector)? as u32;
    let input = unsafe { c::JS_GetPropertyStr(ctx, *config, cs!("input")) };
    let input = js_val_into_bytes(ctx, input)?;

    let output = contract_call::invoke_contract_delegate(delegate.into(), selector, &input)
        .map_err(|err| alloc::format!("{:?}", err))?;

    Ok(js_val_from_bytes(ctx, &output))
}
