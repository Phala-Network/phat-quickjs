use js::{Error as ValueError, FromJsValue};

use super::*;

pub enum OwnedJsValue {
    Undefined,
    Null,
    Exception,
    Other {
        runtime: Weak<JsEngine>,
        value: c::JSValue,
    },
}

impl TryFrom<js::Value> for OwnedJsValue {
    type Error = ValueError;

    fn try_from(v: js::Value) -> Result<Self, Self::Error> {
        match &v {
            js::Value::Undefined => Ok(OwnedJsValue::Undefined),
            js::Value::Null => Ok(OwnedJsValue::Null),
            js::Value::Exception => Ok(OwnedJsValue::Exception),
            js::Value::Other { ctx, value } => {
                let runtime = js_context_get_runtime(&ctx).ok_or(ValueError::RuntimeDropped)?;
                let v = unsafe { c::JS_DupValue(ctx.as_ptr(), *value) };
                Ok(OwnedJsValue::from_raw(v, Rc::downgrade(&runtime)))
            }
        }
    }
}

impl FromJsValue for OwnedJsValue {
    fn from_js_value(value: js::Value) -> Result<Self, ValueError> {
        Self::try_from(value)
    }
}

impl TryFrom<OwnedJsValue> for js::Value {
    type Error = ValueError;

    fn try_from(v: OwnedJsValue) -> Result<Self, Self::Error> {
        match &v {
            OwnedJsValue::Undefined => Ok(js::Value::Undefined),
            OwnedJsValue::Null => Ok(js::Value::Null),
            OwnedJsValue::Exception => Ok(js::Value::Exception),
            OwnedJsValue::Other { runtime, value } => {
                let engine = runtime.upgrade().ok_or(ValueError::RuntimeDropped)?;
                Ok(js::Value::new_cloned(&engine.ctx, *value))
            }
        }
    }
}

impl Drop for OwnedJsValue {
    fn drop(&mut self) {
        match self {
            Self::Undefined | Self::Null | Self::Exception => {}
            Self::Other { runtime, value } => {
                if let Some(runtime) = runtime.upgrade() {
                    runtime.free_value(*value);
                }
            }
        }
    }
}

impl core::fmt::Debug for OwnedJsValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.to_js_value() {
            Some(value) => write!(f, "{}", value),
            None => write!(f, "<dropped>"),
        }
    }
}

impl OwnedJsValue {
    pub fn undefined() -> Self {
        Self::Undefined
    }
    pub fn null() -> Self {
        Self::Null
    }
    pub fn exception() -> Self {
        Self::Exception
    }
    pub fn from_raw(value: c::JSValue, runtime: Weak<JsEngine>) -> Self {
        Self::Other { value, runtime }
    }
    pub fn dup(&self) -> Option<Self> {
        match self {
            Self::Undefined => Some(Self::Undefined),
            Self::Null => Some(Self::Null),
            Self::Exception => Some(Self::Exception),
            Self::Other { runtime, value } => {
                let runtime = runtime.upgrade()?;
                Some(runtime.dup_value(*value))
            }
        }
    }
    pub fn value(&self) -> &c::JSValue {
        match self {
            Self::Undefined => &c::JS_UNDEFINED,
            Self::Null => &c::JS_NULL,
            Self::Exception => &c::JS_EXCEPTION,
            Self::Other { value, .. } => value,
        }
    }

    pub fn is_undefined(&self) -> bool {
        unsafe { c::JS_IsUndefined(*self.value()) == 1 }
    }

    pub fn to_js_value(&self) -> Option<js::Value> {
        self.dup()?.try_into().ok()
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
