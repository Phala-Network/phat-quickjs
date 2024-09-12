use super::*;

use log::{info, warn, debug, error};
use qjs_extensions::repr;

use crate::service::ServiceRef;

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("print", print)?;
    Ok(())
}

#[js::host_call(with_context)]
fn print(service: ServiceRef, _this: js::Value, level: u32, args: Vec<js::Value>, config: Option<repr::ReprConfig>) {
    let is_sandbox = service.is_sandbox();
    let buf = repr::print(&args, &config.unwrap_or_default());
    let buf = buf.trim_end();
    if buf.is_empty() {
        js_log(level, "", is_sandbox);
    } else {
        let buf = &buf[..20480.min(buf.len())];
        for line in buf.lines() {
            js_log(level, line, is_sandbox);
        }
    }
    if buf.len() > 20480 {
        js_log(level, "<...>", is_sandbox);
    }
}

pub fn js_log(level: u32, msg: &str, is_sandbox: bool) {
    let target = match is_sandbox {
        true => "js::console::sandbox",
        false => "js::console",
    };
    match level {
        1 => debug!(target: target, "{msg}"),
        2 => info!(target: target, "{msg}"),
        3 => warn!(target: target, "{msg}"),
        _ => error!(target: target, "{msg}"),
    }
}
