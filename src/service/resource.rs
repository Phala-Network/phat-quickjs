use log::info;
use qjs::{Error as ValueError, FromJsValue, Value as JsValue};

use super::*;

pub struct OwnedJsValue {
    runtime: Weak<Runtime>,
    value: c::JSValue,
}

impl TryFrom<JsValue> for OwnedJsValue {
    type Error = ValueError;

    fn try_from(v: JsValue) -> Result<Self, Self::Error> {
        let ctx = v.context()?;
        let runtime =
            js_context_get_runtime(ctx).ok_or(ValueError::Static("Failed to get service"))?;
        let v = unsafe { c::JS_DupValue(ctx, *v.raw_value()) };
        Ok(OwnedJsValue::from_raw(v, Rc::downgrade(&runtime)))
    }
}

impl FromJsValue for OwnedJsValue {
    fn from_js_value(value: JsValue) -> Result<Self, ValueError> {
        Self::try_from(value)
    }
}

impl TryFrom<OwnedJsValue> for JsValue {
    type Error = ValueError;

    fn try_from(v: OwnedJsValue) -> Result<Self, Self::Error> {
        let ctx = v
            .runtime
            .upgrade()
            .ok_or(ValueError::Static("Runtime has been dropped"))?
            .ctx;
        Ok(JsValue::new_cloned(ctx, v.value))
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

impl core::fmt::Debug for OwnedJsValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match JsValue::try_from(self.clone()) {
            Ok(value) => write!(f, "{}", value),
            Err(_) => write!(f, "<dropped>"),
        }
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

    pub fn is_undefined(&self) -> bool {
        unsafe { c::JS_IsUndefined(self.value) == 1 }
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
