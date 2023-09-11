use qjsbind as js;
use alloc::vec::Vec;
use alloc::string::{String, ToString};

pub fn setup(obj: &js::Value) -> js::Result<()> {
    obj.define_property_fn("print", print)?;
    obj.define_property_fn("repr", to_debug_representation)?;
    Ok(())
}

#[js::host_call]
fn print(depth: u8, level: u8, args: Vec<js::Value>) {
    let mut buf = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i != 0 {
            buf.push(' ');
        }
        if depth == 0 {
            buf.push_str(&arg.to_string());
        } else {
            let mut buf2 = String::new();
            js::recursive_to_string(arg, depth, false, &mut buf2);
            buf.push_str(&buf2);
        }
    }
    let buf = buf.trim_end();
    if buf.is_empty() {
        log_str(level, "");
    } else {
        for line in buf.lines() {
            log_str(level, line);
        }
    }
}

fn log_str(level: u8, msg: &str) {
    match level {
        1 => pink::debug!("JS: {}", msg),
        2 => pink::info!("JS: {}", msg),
        3 => pink::warn!("JS: {}", msg),
        4 => pink::error!("JS: {}", msg),
        _ => {
            pink::error!("JS: {}", msg);
        }
    }
}

#[js::host_call]
fn to_debug_representation(obj: js::Value, level: Option<u8>) -> String {
    let level = level.unwrap_or(5);
    let mut buf = String::new();
    js::recursive_to_string(&obj, level, true, &mut buf);
    buf
}
