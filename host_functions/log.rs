use qjsbind as js;
use alloc::vec::Vec;
use alloc::string::{String, ToString};

pub fn setup(obj: &js::Value) -> js::Result<()> {
    obj.define_property_fn("log", log)?;
    Ok(())
}

#[js::host_call]
fn log(level: u8, args: Vec<js::Value>) {
    let mut buf = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i != 0 {
            buf.push(' ');
        }
        buf.push_str(&arg.to_string());
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
        1 => pink::debug!("{}", msg),
        2 => pink::info!("{}", msg),
        3 => pink::warn!("{}", msg),
        4 => pink::error!("{}", msg),
        _ => {
            pink::error!("{}", msg);
        }
    }
}
