use log::info;
use qjs_sys::convert::DecodeFromJSValue;

use super::*;

pub struct OwnedJsValue {
    runtime: Weak<Runtime>,
    value: c::JSValue,
}

impl DecodeFromJSValue for OwnedJsValue {
    fn decode(ctx: *mut c::JSContext, v: c::JSValue) -> std::result::Result<Self, &'static str>
    where
        Self: Sized,
    {
        let runtime = js_context_get_runtime(ctx).ok_or("Failed to get service")?;
        let v = unsafe { c::JS_DupValue(ctx, v) };
        Ok(OwnedJsValue::from_raw(v, Rc::downgrade(&runtime)))
    }
}

impl Drop for OwnedJsValue {
    fn drop(&mut self) {
        let Some(runtime) = self.runtime.upgrade() else {
            info!("Can not free JSValue. The service has been dropped");
            return;
        };
        runtime.free_value(self.value);
    }
}

impl Clone for OwnedJsValue {
    fn clone(&self) -> Self {
        self.dup()
            .expect("Failed to dup JsValue, the service has been dropped")
    }
}

impl OwnedJsValue {
    pub fn from_raw(value: c::JSValue, runtime: Weak<Runtime>) -> Self {
        Self { value, runtime }
    }

    pub fn dup(&self) -> Option<Self> {
        let runtime = self.runtime.upgrade()?;
        Some(runtime.dup_value(self.value))
    }

    pub fn value(&self) -> &c::JSValue {
        &self.value
    }
}

pub struct Resource {
    pub js_value: OwnedJsValue,
    _cancel_token: Option<Box<dyn Any>>,
}

impl Resource {
    pub fn new(js_value: OwnedJsValue, cancel_token: Option<Box<dyn Any>>) -> Self {
        Self {
            js_value,
            _cancel_token: cancel_token,
        }
    }
}
