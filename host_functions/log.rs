use alloc::vec::Vec;
use qjs_extensions::repr;
use qjsbind as js;

pub fn setup(obj: &js::Value) -> js::Result<()> {
    obj.define_property_fn("print", print)?;
    Ok(())
}

#[js::host_call]
fn print(level: u8, args: Vec<js::Value>, config: Option<repr::ReprConfig>) {
    let buf = repr::print(&args, &config.unwrap_or_default());
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
