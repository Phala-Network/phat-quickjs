use super::*;

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("print", print)?;
    Ok(())
}

#[js::host_call(with_context)]
fn print(service: ServiceRef, _this: js::Value, level: u8, fd: u32, args: Vec<js::Value>) {
    let mut buf = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i != 0 {
            buf.push_str(" ");
        }
        let mut buf2 = String::new();
        js::recursive_to_string(arg, level, false, &mut buf2);
        buf.push_str(&buf2);
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
