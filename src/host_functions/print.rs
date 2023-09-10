use super::*;

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("print", print)?;
    Ok(())
}

fn recursive_to_string(value: &js::Value, level: u8, escape: bool, buf: &mut String) {
    if value.is_generic_object() {
        if level == 0 {
            buf.push_str("[object Object]");
        } else {
            let mut first = true;
            let Ok(entries) = value.entries() else {
                buf.push_str("[object Object]");
                return;
            };
            buf.push_str("{");
            for r in entries {
                let Ok((key, value)) = r else {
                    continue;
                };
                if first {
                    first = false;
                } else {
                    buf.push_str(", ");
                }
                buf.push_str(&key.to_string());
                buf.push_str(": ");
                recursive_to_string(&value, level - 1, true, buf);
            }
            buf.push_str("}");
        }
        return;
    }
    if escape && value.is_string() {
        // print escaped string
        buf.push('"');
        for c in value.to_string().chars() {
            match c {
                '"' => buf.push_str("\\\""),
                '\\' => buf.push_str("\\\\"),
                '\n' => buf.push_str("\\n"),
                '\r' => buf.push_str("\\r"),
                '\t' => buf.push_str("\\t"),
                '\u{0008}' => buf.push_str("\\b"),
                '\u{000C}' => buf.push_str("\\f"),
                '\u{000B}' => buf.push_str("\\v"),
                '\u{0007}' => buf.push_str("\\a"),
                '\u{0000}' => buf.push_str("\\0"),
                _ => buf.push(c),
            }
        }
        buf.push('"');
        return;
    }
    buf.push_str(&value.to_string());
}

#[js::host_call(with_context)]
fn print(service: ServiceRef, _this: js::Value, level: u8, fd: u32, args: Vec<js::Value>) {
    let mut buf = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i != 0 {
            buf.push_str(" ");
        }
        let mut buf2 = String::new();
        recursive_to_string(arg, level, false, &mut buf2);
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
