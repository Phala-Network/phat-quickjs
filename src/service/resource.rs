use qjs::{Error as ValueError, FromJsValue, Value as JsValue};

use super::*;

pub enum OwnedJsValue {
    Undefined,
    Null,
    Exception,
    Other {
        runtime: Weak<Runtime>,
        value: c::JSValue,
    },
}

impl TryFrom<JsValue> for OwnedJsValue {
    type Error = ValueError;

    fn try_from(v: JsValue) -> Result<Self, Self::Error> {
        match v {
            JsValue::Undefined => Ok(OwnedJsValue::Undefined),
            JsValue::Null => Ok(OwnedJsValue::Null),
            JsValue::Exception => Ok(OwnedJsValue::Exception),
            JsValue::Other { ctx, value } => {
                let runtime =
                    js_context_get_runtime(ctx).ok_or(ValueError::Static("service gone"))?;
                let v = unsafe { c::JS_DupValue(ctx.as_ptr(), value) };
                Ok(OwnedJsValue::from_raw(v, Rc::downgrade(&runtime)))
            }
        }
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
        match &v {
            OwnedJsValue::Undefined => Ok(JsValue::Undefined),
            OwnedJsValue::Null => Ok(JsValue::Null),
            OwnedJsValue::Exception => Ok(JsValue::Exception),
            OwnedJsValue::Other { runtime, value } => {
                let ctx = runtime
                    .upgrade()
                    .ok_or(ValueError::Static("Runtime has been dropped"))?
                    .ctx;
                Ok(JsValue::new_cloned(ctx, *value))
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
    pub fn undefined() -> Self {
        Self::Undefined
    }
    pub fn null() -> Self {
        Self::Null
    }
    pub fn exception() -> Self {
        Self::Exception
    }
    pub fn from_raw(value: c::JSValue, runtime: Weak<Runtime>) -> Self {
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
