use super::*;
use log::info;
use qjs::{host_call, Value as JsValue};

pub(crate) fn setup(ns: &JsValue) -> Result<()> {
    ns.set_property_fn("marker", marker)?;
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

#[host_call]
fn marker(service: ServiceRef, _this: JsValue, tag: String) -> JsValue {
    JsValue::new_opaque_object(service.raw_ctx(), Marker { tag })
}
