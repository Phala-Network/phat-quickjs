use alloc::{
    boxed::Box,
    collections::BTreeMap,
    ffi::CString,
    rc::{Rc, Weak},
    string::ToString,
};
use core::{any::Any, cell::RefCell, ffi::CStr};
use log::error;

use anyhow::Result;
use qjs_sys::{c, convert::DecodeFromJSValue, JsCode, Output};

pub(crate) type ServiceRef = Rc<Service>;
pub(crate) type ServiceWeakRef = Weak<Service>;

pub struct Service {
    name: String,
    runtime: *mut c::JSRuntime,
    ctx: *mut c::JSContext,
    state: RefCell<ServiceState>,
}

#[derive(Default)]
struct ServiceState {
    next_resource_id: u64,
    recources: BTreeMap<u64, Resource>,
}

impl Service {
    pub fn new(name: &str, weak_self: ServiceWeakRef) -> Self {
        let runtime = unsafe { c::JS_NewRuntime() };
        let ctx = unsafe { c::JS_NewContext(runtime) };
        let bootcode = JsCode::Bytecode(bootcode::BOOT_CODE);

        qjs_sys::ctx_init(ctx);
        qjs_sys::ctx_eval(ctx, bootcode).expect("Failed to eval bootcode");

        let boxed_self = Box::into_raw(Box::new(weak_self));
        unsafe { c::JS_SetContextOpaque(ctx, boxed_self as *mut _) };
        let state = RefCell::new(ServiceState::default());
        Self {
            name: name.into(),
            runtime,
            ctx,
            state,
        }
    }

    pub fn new_ref(name: &str) -> ServiceRef {
        Rc::new_cyclic(|weak_self| Service::new(name, weak_self.clone()))
    }

    pub fn weak_self(&self) -> ServiceWeakRef {
        unsafe {
            let ptr = c::JS_GetContextOpaque(self.ctx) as *mut ServiceWeakRef;
            (*ptr).clone()
        }
    }

    pub fn exec_script(&self, script: &str) -> Result<Output, String> {
        let script = CString::new(script).or(Err("Failed to convert source to CString"))?;
        let js_code = qjs_sys::JsCode::Source(script.as_c_str());
        qjs_sys::ctx_eval(self.ctx, js_code)
    }

    pub fn dup_value(&self, value: c::JSValue) -> OwnedJsValue {
        let value = unsafe { c::JS_DupValue(self.ctx, value) };
        OwnedJsValue {
            service: self.weak_self(),
            value,
        }
    }

    pub fn free_value(&self, value: c::JSValue) {
        unsafe { c::JS_FreeValue(self.ctx, value) };
    }

    pub fn push_resource(&self, resource: Resource) -> u64 {
        let mut state = self.state.borrow_mut();
        let id = state.next_resource_id;
        state.next_resource_id += 1;
        state.recources.insert(id, resource);
        id
    }

    pub fn get_resource_value(&self, id: u64) -> Option<OwnedJsValue> {
        let state = self.state.borrow();
        let value = state.recources.get(&id)?.js_value.dup()?;
        Some(value)
    }

    pub fn remove_resource(&self, id: u64) -> Option<Resource> {
        let mut state = self.state.borrow_mut();
        state.recources.remove(&id)
    }

    pub fn call_function(&self, func: c::JSValue, args: &[c::JSValue]) -> Result<c::JSValue> {
        let this = c::JS_UNDEFINED;
        let ret = unsafe {
            let len = args.len();
            let args_len = len as core::ffi::c_int;
            let args = args.as_ptr();
            let args = args as *mut c::JSValue;
            c::JS_Call(self.ctx, func, this, args_len, args)
        };
        if ret == c::JS_EXCEPTION {
            let exception = unsafe { c::JS_GetException(self.ctx) };
            let err = qjs_sys::ctx_to_string(self.ctx, exception);
            anyhow::bail!("Failed to call function: {err}");
        }
        Ok(ret)
    }
}

pub fn js_context_get_service(ctx: *mut c::JSContext) -> Option<ServiceWeakRef> {
    unsafe {
        let name = c::JS_GetContextOpaque(ctx) as *mut ServiceWeakRef;
        if name.is_null() {
            return None;
        }
        Some((*name).clone())
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        unsafe {
            let pname = c::JS_GetContextOpaque(self.ctx) as *mut ServiceWeakRef;
            drop(Box::from_raw(pname));
            c::JS_FreeContext(self.ctx);
            c::JS_FreeRuntime(self.runtime);
        }
    }
}

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
