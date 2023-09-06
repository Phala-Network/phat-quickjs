use qjsbind::{Value as JsValue, host_call, Result};

pub fn setup(obj: &JsValue) -> Result<()> {
    obj.define_property_fn("log", log)?;
    Ok(())
}

#[host_call]
fn log(level: u8, args: Vec<JsValue>) {
    let mut buf = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i != 0 {
            buf.push_str(" ");
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
