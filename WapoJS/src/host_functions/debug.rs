use super::*;
use log::info;

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("marker", marker)?;
    Ok(())
}

struct Marker {
    tag: String,
}
impl Drop for Marker {
    fn drop(&mut self) {
        info!(target: "js::debug", "dropping marker: {}", self.tag);
    }
}

#[js::host_call(with_context)]
fn marker(service: ServiceRef, _this: js::Value, tag: String) -> js::Value {
    js::Value::new_opaque_object(service.context(), Some("Marker"), Marker { tag })
}
