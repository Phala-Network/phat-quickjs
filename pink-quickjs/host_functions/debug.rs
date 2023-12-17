use super::*;
use alloc::string::String;

pub(crate) fn setup(ns: &js::Value) -> js::Result<()> {
    ns.define_property_fn("marker", marker)?;
    Ok(())
}

struct Marker {
    tag: String,
}
impl Drop for Marker {
    fn drop(&mut self) {
        pink::info!("Dropping marker: {}", self.tag);
    }
}

#[js::host_call(with_context)]
fn marker(context: js::Context, _this: js::Value, tag: String) -> js::Value {
    js::Value::new_opaque_object(&context, Marker { tag })
}
