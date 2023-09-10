use super::*;
use log::info;

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("marker", marker)?;
    ns.define_property_fn("repr", to_debug_representation)?;
    Ok(())
}

struct Marker {
    tag: String,
}
impl Drop for Marker {
    fn drop(&mut self) {
        info!("Dropping marker: {}", self.tag);
    }
}

#[js::host_call(with_context)]
fn marker(service: ServiceRef, _this: js::Value, tag: String) -> js::Value {
    js::Value::new_opaque_object(service.raw_ctx(), Marker { tag })
}

#[js::host_call]
fn to_debug_representation(obj: js::Value, level: Option<u8>) -> Result<String> {
    let level = level.unwrap_or(5);
    let mut buf = String::new();
    js::recursive_to_string(&obj, level, true, &mut buf);
    Ok(buf)
}
