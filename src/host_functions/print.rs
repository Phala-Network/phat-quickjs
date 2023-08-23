use super::*;

pub(crate) fn setup(ns: &JsValue) -> Result<()> {
    ns.set_property_fn("print", print)?;
    Ok(())
}

#[qjs::host_call]
fn print(service: ServiceRef, _this: JsValue, fd: u32, args: Vec<JsValue>) {
    let mut buf = String::new();
    for (i, arg) in args[1..].iter().enumerate() {
        if i != 0 {
            buf.push_str(" ");
        }
        buf.push_str(&arg.to_string());
    }
    let buf = buf.trim_end();
    if buf.is_empty() {
        service.js_log(fd, "");
    } else {
        for line in buf.lines() {
            service.js_log(fd, line);
        }
    }
}
