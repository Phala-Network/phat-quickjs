use super::*;
use qjs::{host_call, Value as JsValue};

pub(crate) fn setup(ns: &JsValue) -> Result<()> {
    ns.set_property_fn("rcMark", rc_mark)?;
    Ok(())
}

struct Marker {
    tag: String,
}
impl Drop for Marker {
    fn drop(&mut self) {
        println!("Dropping marker: {}", self.tag);
    }
}

#[host_call]
fn rc_mark(service: ServiceRef, _this: JsValue, tag: String) -> JsValue {
    JsValue::new_opaque_object(service.raw_ctx(), Marker { tag })
}
