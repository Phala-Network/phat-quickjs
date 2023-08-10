use super::*;

use anyhow::Context;
use qjs_sys as qjs;

pub(super) fn print(
    service: ServiceRef,
    ctx: *mut c::JSContext,
    args: &[c::JSValueConst],
) -> Result<JsValue> {
    let mut buf = String::new();
    if args.len() < 1 {
        return Err("print: expected at least one argument").anyhow();
    }
    let fd: u32 = DecodeFromJSValue::decode(ctx, args[0])
        .anyhow()
        .context("print: Failed to decode fd")?;
    for (i, arg) in args[1..].iter().enumerate() {
        if i != 0 {
            buf.push_str(" ");
        }
        qjs::ctx_to_str(ctx, *arg, |s| buf.push_str(s));
    }
    let buf = buf.trim_end();
    if buf.is_empty() {
        service.js_log(fd, "");
    } else {
        for line in buf.lines() {
            service.js_log(fd, line);
        }
    }
    return Ok(JsValue::Undefined);
}
