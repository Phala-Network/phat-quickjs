use super::*;

use qjs_extensions::repr;

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("print", print)?;
    Ok(())
}

#[js::host_call(with_context)]
fn print(
    service: ServiceRef,
    _this: js::Value,
    level: u32,
    args: Vec<js::Value>,
    config: Option<repr::ReprConfig>,
) {
    let buf = repr::print(&args, &config.unwrap_or_default());
    let buf = buf.trim_end();
    if buf.is_empty() {
        service.js_log(level, "");
    } else {
        let buf = &buf[..2048.min(buf.len())];
        for line in buf.lines() {
            service.js_log(level, line);
        }
    }
    if buf.len() > 2048 {
        service.js_log(level, "<...>");
    }
}
