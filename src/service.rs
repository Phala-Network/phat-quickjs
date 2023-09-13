use alloc::{
    boxed::Box,
    collections::BTreeMap,
    rc::{Rc, Weak},
};
use core::{any::Any, cell::RefCell, ops::Deref};
use log::{debug, error, info, warn};
use std::future::Future;

use crate::host_functions::setup_host_functions;
use anyhow::Result;
use js::{c, Code, Error as ValueError, ToArgs};
use tokio::sync::broadcast;

mod resource;

pub(crate) use resource::{OwnedJsValue, Resource};

#[derive(Clone)]
pub(crate) struct ServiceRef(Rc<Service>);
#[derive(Clone)]
pub(crate) struct ServiceWeakRef(Weak<Service>);

impl Deref for ServiceRef {
    type Target = Rc<Service>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for ServiceWeakRef {
    type Target = Weak<Service>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ServiceWeakRef {
    pub fn upgrade(&self) -> Option<ServiceRef> {
        self.0.upgrade().map(ServiceRef)
    }
}

impl TryFrom<js::Context> for ServiceRef {
    type Error = anyhow::Error;

    fn try_from(ctx: js::Context) -> Result<Self, Self::Error> {
        let weak_srv = js_context_get_service(&ctx)
            .ok_or(anyhow::anyhow!("Failed to get service from context"))?;
        weak_srv
            .upgrade()
            .ok_or(anyhow::anyhow!("Service has been dropped"))
    }
}

pub struct JsEngine {
    pub ctx: js::Context,
    runtime: js::Runtime,
    weak_self: Weak<JsEngine>,
}

impl JsEngine {
    pub fn free_value(&self, value: c::JSValue) {
        unsafe { c::JS_FreeValue(self.ctx.as_ptr(), value) };
    }

    pub fn dup_value(&self, value: c::JSValue) -> OwnedJsValue {
        let value = unsafe { c::JS_DupValue(self.ctx.as_ptr(), value) };
        OwnedJsValue::from_raw(value, self.weak_self.clone())
    }

    pub fn to_js_value(&self, owned: &OwnedJsValue) -> js::Value {
        js::Value::new_cloned(&self.ctx, *owned.value())
    }

    pub fn to_owned_value(&self, js_value: &js::Value) -> OwnedJsValue {
        self.dup_value(*js_value.raw_value())
    }

    pub fn exec_pending_jobs(&self) {
        loop {
            match self.runtime.exec_pending_jobs() {
                Ok(0) => break,
                Ok(cnt) => {
                    debug!("Executed {} pending jobs", cnt);
                }
                Err(err) => {
                    // TODO.kevin: should we continue?
                    error!("Failed to execute pending jobs: {err}");
                    break;
                }
            }
        }
    }
}

pub struct Service {
    runtime: Rc<JsEngine>,
    state: RefCell<ServiceState>,
}

struct ServiceState {
    next_resource_id: u64,
    recources: BTreeMap<u64, Resource>,
    http_listener: Option<OwnedJsValue>,
    done_tx: broadcast::Sender<()>,
}

impl ServiceState {
    fn take_next_resource_id(&mut self) -> u64 {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        id
    }
}

impl Default for ServiceState {
    fn default() -> Self {
        Self {
            next_resource_id: Default::default(),
            recources: Default::default(),
            http_listener: Default::default(),
            done_tx: broadcast::channel(1).0,
        }
    }
}

pub fn ctx_init(ctx: &js::Context) {
    unsafe {
        let ctx = ctx.as_ptr();
        #[cfg(feature = "stream")]
        c::js_stream_init(ctx);
        c::js_blob_init(ctx);
        c::js_opaque_class_init(ctx)
    };
}

impl Service {
    pub(crate) fn new(weak_self: ServiceWeakRef) -> Self {
        let runtime = js::Runtime::new();
        let ctx = runtime.new_context();
        let boxed_self = Box::into_raw(Box::new(weak_self));
        unsafe { c::JS_SetContextOpaque(ctx.as_ptr(), boxed_self as *mut _) };
        ctx_init(&ctx);
        setup_host_functions(&ctx).expect("Failed to setup host functions");
        let bootcode = Code::Bytecode(bootcode::BOOT_CODE);
        ctx.eval(&bootcode).expect("Failed to eval bootcode");
        let state = RefCell::new(ServiceState::default());
        Self {
            runtime: Rc::new_cyclic(|weak_self| JsEngine {
                runtime,
                ctx,
                weak_self: weak_self.clone(),
            }),
            state,
        }
    }

    pub(crate) fn new_ref() -> ServiceRef {
        ServiceRef(Rc::new_cyclic(|weak_self| {
            Service::new(ServiceWeakRef(weak_self.clone()))
        }))
    }

    pub(crate) fn weak_self(&self) -> ServiceWeakRef {
        unsafe {
            let ptr = c::JS_GetContextOpaque(self.context().as_ptr()) as *mut ServiceWeakRef;
            (*ptr).clone()
        }
    }

    // TODO.kevin: rename to ctx
    pub fn context(&self) -> &js::Context {
        &self.runtime.ctx
    }

    pub fn runtime(&self) -> Rc<JsEngine> {
        self.runtime.clone()
    }

    pub fn exec_script(&self, script: &str) -> Result<OwnedJsValue, String> {
        self.eval(Code::Source(script))
    }

    pub fn exec_bytecode(&self, script: &[u8]) -> Result<OwnedJsValue, String> {
        self.eval(Code::Bytecode(script))
    }

    pub fn eval(&self, code: Code) -> Result<OwnedJsValue, String> {
        let result = js::eval(self.context(), &code)
            .map(|value| value.try_into().map_err(|err: ValueError| err.to_string()))?;
        self.runtime.exec_pending_jobs();
        result
    }

    pub fn call_function(&self, func: js::Value, args: impl ToArgs) -> Result<js::Value> {
        let ctx = self.context();
        let mut args = args.to_raw_args(ctx)?;
        let func = *func.raw_value();
        let this = c::JS_UNDEFINED;
        let ret = unsafe {
            let len = args.len();
            let args_len = len as core::ffi::c_int;
            let args = args.as_mut_ptr();
            c::JS_Call(ctx.as_ptr(), func, this, args_len, args)
        };
        if c::is_exception(ret) {
            let err = self.context().get_exception_str();
            anyhow::bail!("Failed to call function: {err}");
        }
        self.runtime.exec_pending_jobs();
        Ok(js::Value::new_moved(self.context(), ret))
    }

    pub fn push_resource(&self, resource: Resource) -> u64 {
        let mut state = self.state.borrow_mut();
        let id = state.take_next_resource_id();
        state.recources.insert(id, resource);
        id
    }

    pub fn get_resource_value(&self, id: u64) -> Option<js::Value> {
        let state = self.state.borrow();
        Some(self.to_js_value(&state.recources.get(&id)?.js_value))
    }

    pub fn remove_resource(&self, id: u64) -> Option<Resource> {
        debug!("Removing resource {id}");
        let mut state = self.state.borrow_mut();
        let was_empty = state.recources.is_empty();
        let res = state.recources.remove(&id);
        if !was_empty && state.recources.is_empty() {
            let _ = state.done_tx.send(());
        }
        res
    }

    pub(crate) fn spawn<Fut, FutGen, Args>(
        &self,
        js_callback: OwnedJsValue,
        fut_gen: FutGen,
        args: Args,
    ) -> u64
    where
        Fut: Future<Output = ()> + 'static,
        Args: 'static,
        FutGen: FnOnce(ServiceWeakRef, u64, Args) -> Fut + 'static,
    {
        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
        let res = Resource::new(js_callback, Some(Box::new(cancel_tx)));
        let id = self.push_resource(res);
        let weak_service = self.weak_self();
        let _handle = crate::runtime::spawn(async move {
            tokio::select! {
                _ = fut_gen(weak_service.clone(), id, args) => {
                }
                _ = cancel_rx => {
                }
            }
            close(weak_service, id);
        });
        id
    }
    pub fn js_log(&self, level: u32, msg: &str) {
        match level {
            1 => debug!("JS:[{level}]|  {}", msg),
            2 => info!("JS:[{level}]|  {}", msg),
            3 => warn!("JS:[{level}]|  {}", msg),
            _ => error!("JS:[{level}]|  {}", msg),
        }
    }

    pub async fn wait_for_tasks(&self) {
        if self.state.borrow().recources.len() == 0 {
            return;
        }
        let mut rx = self.state.borrow().done_tx.subscribe();
        let _ = rx.recv().await;
    }

    pub fn number_of_tasks(&self) -> usize {
        self.state.borrow().recources.len()
    }

    pub(crate) fn set_http_listener(&self, listener: OwnedJsValue) {
        self.state.borrow_mut().http_listener = Some(listener);
    }

    pub(crate) fn http_listener(&self) -> Option<OwnedJsValue> {
        self.state.borrow().http_listener.as_ref()?.dup()
    }

    pub fn to_js_value(&self, owned: &OwnedJsValue) -> js::Value {
        self.runtime.to_js_value(owned)
    }

    pub fn to_owned_value(&self, js_value: &js::Value) -> OwnedJsValue {
        self.runtime.to_owned_value(js_value)
    }
}

pub(crate) fn close(weak_service: ServiceWeakRef, id: u64) {
    let Some(service) = weak_service.upgrade() else {
        return;
    };
    _ = service.remove_resource(id);
}

pub(crate) fn js_context_get_service(ctx: &js::Context) -> Option<ServiceWeakRef> {
    unsafe {
        let name = c::JS_GetContextOpaque(ctx.as_ptr()) as *mut ServiceWeakRef;
        if name.is_null() {
            return None;
        }
        Some((*name).clone())
    }
}

pub fn js_context_get_runtime(ctx: &js::Context) -> Option<Rc<JsEngine>> {
    Some(js_context_get_service(ctx)?.upgrade()?.runtime())
}

impl Drop for Service {
    fn drop(&mut self) {
        unsafe {
            // release all js resources before destroy the runtime
            *self.state.borrow_mut() = Default::default();
            let pname = c::JS_GetContextOpaque(self.context().as_ptr()) as *mut ServiceWeakRef;
            drop(Box::from_raw(pname));
        }
    }
}
