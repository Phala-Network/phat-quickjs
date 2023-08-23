use alloc::{
    boxed::Box,
    collections::BTreeMap,
    ffi::CString,
    rc::{Rc, Weak},
};
use core::{any::Any, cell::RefCell, ops::Deref};
use log::{debug, error, info};
use std::future::Future;

use crate::host_functions::setup_host_functions;
use anyhow::Result;
use qjs::{c, Error as ValueError, JsCode, ToArgs, Value as JsValue};
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

impl TryFrom<*mut c::JSContext> for ServiceRef {
    type Error = anyhow::Error;

    fn try_from(ctx: *mut c::JSContext) -> Result<Self, Self::Error> {
        let weak_srv = js_context_get_service(ctx)
            .ok_or(anyhow::anyhow!("Failed to get service from context"))?;
        weak_srv
            .upgrade()
            .ok_or(anyhow::anyhow!("Service has been dropped"))
    }
}

pub struct Runtime {
    runtime: *mut c::JSRuntime,
    pub ctx: *mut c::JSContext,
}

impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe {
            c::JS_FreeContext(self.ctx);
            c::JS_FreeRuntime(self.runtime);
        }
    }
}

impl Runtime {
    pub fn free_value(&self, value: c::JSValue) {
        unsafe { c::JS_FreeValue(self.ctx, value) };
    }

    pub fn dup_value(&self, value: c::JSValue) -> OwnedJsValue {
        let value = unsafe { c::JS_DupValue(self.ctx, value) };
        let runtime = js_context_get_runtime(self.ctx)
            .expect("Failed to get service from context, can not dup value");
        OwnedJsValue::from_raw(value, Rc::downgrade(&runtime))
    }

    pub fn exec_pending_jobs(&self) {
        let mut ctx: *mut c::JSContext = core::ptr::null_mut();
        loop {
            let ret = unsafe { c::JS_ExecutePendingJob(self.runtime, &mut ctx) };
            if ret == 0 {
                break;
            }
            if ret < 0 {
                error!(
                    "Failed to execute pending job: {}",
                    qjs::ctx_get_exception_str(ctx)
                );
            }
        }
    }
}

pub struct Service {
    runtime: Rc<Runtime>,
    state: RefCell<ServiceState>,
}

struct ServiceState {
    next_resource_id: u64,
    recources: BTreeMap<u64, Resource>,
    done_tx: broadcast::Sender<()>,
}

impl Default for ServiceState {
    fn default() -> Self {
        Self {
            next_resource_id: Default::default(),
            recources: Default::default(),
            done_tx: broadcast::channel(1).0,
        }
    }
}

pub fn ctx_init(ctx: *mut c::JSContext) {
    unsafe {
        c::js_pink_env_init(ctx);
        #[cfg(feature = "stream")]
        c::js_stream_init(ctx);
        #[cfg(feature = "blob")]
        c::js_blob_init(ctx);
    };
}

impl Service {
    pub(crate) fn new(weak_self: ServiceWeakRef) -> Self {
        let runtime = unsafe { c::JS_NewRuntime() };
        let ctx = unsafe { c::JS_NewContext(runtime) };
        let bootcode = JsCode::Bytecode(bootcode::BOOT_CODE);

        ctx_init(ctx);
        setup_host_functions(ctx).expect("Failed to setup host functions");
        qjs::eval(ctx, &bootcode).expect("Failed to eval bootcode");

        let boxed_self = Box::into_raw(Box::new(weak_self));
        unsafe { c::JS_SetContextOpaque(ctx, boxed_self as *mut _) };
        let state = RefCell::new(ServiceState::default());
        Self {
            runtime: Rc::new(Runtime { runtime, ctx }),
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
            let ptr = c::JS_GetContextOpaque(self.runtime.ctx) as *mut ServiceWeakRef;
            (*ptr).clone()
        }
    }

    pub fn raw_ctx(&self) -> *mut c::JSContext {
        self.runtime.ctx
    }

    pub fn runtime(&self) -> Rc<Runtime> {
        self.runtime.clone()
    }

    pub fn exec_script(&self, script: &str) -> Result<OwnedJsValue, String> {
        let script = CString::new(script).or(Err("Failed to convert source to CString"))?;
        self.eval(JsCode::Source(script.as_c_str()))
    }

    pub fn exec_bytecode(&self, script: &[u8]) -> Result<OwnedJsValue, String> {
        self.eval(JsCode::Bytecode(script))
    }

    pub fn eval(&self, code: JsCode) -> Result<OwnedJsValue, String> {
        let result = qjs::eval(self.runtime.ctx, &code)
            .map(|value| value.try_into().map_err(|err: ValueError| err.to_string()))?;
        self.runtime.exec_pending_jobs();
        result
    }

    pub fn call_function(&self, func: JsValue, args: impl ToArgs) -> Result<JsValue> {
        let ctx = self.raw_ctx();
        let mut args = args.to_raw_args(ctx)?;
        let func = *func.raw_value();
        let this = c::JS_UNDEFINED;
        let ret = unsafe {
            let len = args.len();
            let args_len = len as core::ffi::c_int;
            let args = args.as_mut_ptr();
            c::JS_Call(ctx, func, this, args_len, args)
        };
        if c::is_exception(ret) {
            let err = qjs::ctx_get_exception_str(self.runtime.ctx);
            anyhow::bail!("Failed to call function: {err}");
        }
        self.runtime.exec_pending_jobs();
        Ok(JsValue::new_moved(self.runtime.ctx, ret))
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
    pub fn js_log(&self, fd: u32, msg: &str) {
        if fd == 1 {
            info!("JS:[{fd}]|  {}", msg);
        } else if fd == 2 {
            error!("JS:[{fd}]|  {}", msg);
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
}

pub(crate) fn close(weak_service: ServiceWeakRef, id: u64) {
    let Some(service) = weak_service.upgrade() else {
        return;
    };
    _ = service.remove_resource(id);
}

pub(crate) fn js_context_get_service(ctx: *mut c::JSContext) -> Option<ServiceWeakRef> {
    unsafe {
        let name = c::JS_GetContextOpaque(ctx) as *mut ServiceWeakRef;
        if name.is_null() {
            return None;
        }
        Some((*name).clone())
    }
}

pub fn js_context_get_runtime(ctx: *mut c::JSContext) -> Option<Rc<Runtime>> {
    Some(js_context_get_service(ctx)?.upgrade()?.runtime())
}

impl Drop for Service {
    fn drop(&mut self) {
        unsafe {
            self.state.borrow_mut().recources.clear();
            let pname = c::JS_GetContextOpaque(self.runtime.ctx) as *mut ServiceWeakRef;
            drop(Box::from_raw(pname));
        }
    }
}
