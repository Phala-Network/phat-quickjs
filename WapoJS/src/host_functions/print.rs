use super::*;

use log::{info, warn, debug, error};
use qjs_extensions::repr;

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("print", print)?;
    Ok(())
}

#[js::host_call]
fn print(level: u32, args: Vec<js::Value>, config: Option<repr::ReprConfig>) {
    let buf = repr::print(&args, &config.unwrap_or_default());
    let buf = buf.trim_end();
    if buf.is_empty() {
        js_log(level, "");
    } else {
        let buf = &buf[..2048.min(buf.len())];
        for line in buf.lines() {
            js_log(level, line);
        }
    }
    if buf.len() > 2048 {
        js_log(level, "<...>");
    }
}

pub fn js_log(level: u32, msg: &str) {
    match level {
        1 => debug!("JS: {}", msg),
        2 => info!("JS: {}", msg),
        3 => warn!("JS: {}", msg),
        _ => error!("JS: {}", msg),
    }
}
