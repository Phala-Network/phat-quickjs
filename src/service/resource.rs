use super::*;

pub struct OwnedJsValue {
    service: ServiceWeakRef,
    value: c::JSValue,
}

impl Drop for OwnedJsValue {
    fn drop(&mut self) {
        let Some(service) = self.service.upgrade() else {
        return;
    };
        service.free_value(self.value);
    }
}

impl Clone for OwnedJsValue {
    fn clone(&self) -> Self {
        self.dup()
            .expect("Failed to dup JsValue, the service has been dropped")
    }
}

impl OwnedJsValue {
    pub fn new(value: c::JSValue, service: ServiceWeakRef) -> Self {
        Self { value, service }
    }

    pub fn dup(&self) -> Option<Self> {
        let service = self.service.upgrade()?;
        Some(service.dup_value(self.value))
    }

    pub fn value(&self) -> &c::JSValue {
        &self.value
    }
}

pub struct Resource {
    pub js_value: OwnedJsValue,
    cancel_token: Option<Box<dyn Any>>,
}

impl Resource {
    pub fn new(js_value: OwnedJsValue, cancel_token: Option<Box<dyn Any>>) -> Self {
        Self {
            js_value,
            cancel_token,
        }
    }
}
