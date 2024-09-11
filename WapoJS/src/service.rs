use alloc::{
    boxed::Box,
    collections::BTreeMap,
    rc::{Rc, Weak},
};
use core::{any::Any, cell::RefCell, ops::Deref, time::Duration};
use log::{debug, error};
use std::ffi::{c_int, c_void, CStr};
use std::{future::Future, sync::Mutex};

use crate::{host_functions::setup_host_functions, runtime};
use anyhow::{Context, Result};
use js::{c, Code, EngineConfig, Error as ValueError, ToArgs};
use tokio::sync::{broadcast, oneshot};

mod resource;

pub(crate) use resource::{OwnedJsValue, Resource};

#[derive(Clone)]
pub struct ServiceRef(Rc<Service>);
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
    type Error = ValueError;

    fn try_from(ctx: js::Context) -> Result<Self, Self::Error> {
        js_context_get_service(&ctx)
            .context("service not found")?
            .upgrade()
            .context("runtime has been dropped")
    }
}

pub struct JsEngine {
    pub ctx: js::Context,
    runtime: js::Runtime,
    weak_self: Weak<JsEngine>,
    last_error: Mutex<Option<String>>,
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

    pub fn take_last_error(&self) -> Option<String> {
        self.last_error.lock().unwrap().take()
    }

    pub fn exec_pending_jobs(&self) {
        let _ = self.take_last_error();
        loop {
            match self.runtime.exec_pending_jobs() {
                Ok(0) => break,
                Ok(cnt) => {
                    debug!(target: "js::rt", "executed {cnt} pending jobs");
                }
                Err(err) => {
                    error!(target: "js::rt", "uncatched error: {err}");
                    *self.last_error.lock().unwrap() = Some(err);
                    continue;
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct ServiceConfig {
    pub engine_config: EngineConfig,
    pub is_sandbox: bool,
    pub worker_secret: String,
}

pub struct Service {
    runtime: Rc<JsEngine>,
    state: RefCell<ServiceState>,
    config: ServiceConfig,
    unhandled_rejection_str: RefCell<Option<String>>,
}

struct ServiceState {
    next_resource_id: u64,
    recources: BTreeMap<u64, Resource>,
    http_listener: Option<OwnedJsValue>,
    query_listener: Option<OwnedJsValue>,
    done_tx: broadcast::Sender<()>,
}

impl ServiceState {
    fn take_next_resource_id(&mut self) -> u64 {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        id
    }
    fn is_empty(&self) -> bool {
        self.recources.is_empty() && self.http_listener.is_none() && self.query_listener.is_none()
    }
}

impl Default for ServiceState {
    fn default() -> Self {
        Self {
            next_resource_id: Default::default(),
            recources: Default::default(),
            http_listener: Default::default(),
            query_listener: Default::default(),
            done_tx: broadcast::channel(1).0,
        }
    }
}

impl Service {
    pub(crate) fn new(weak_self: ServiceWeakRef, config: ServiceConfig) -> Self {
        let runtime = js::Runtime::new(&config.engine_config);
        let ctx = runtime.new_context();
        let boxed_self = Box::into_raw(Box::new(weak_self.clone()));
        unsafe { c::JS_SetContextOpaque(ctx.as_ptr(), boxed_self as *mut _) };

        extern "C" fn promise_rejection_tracker(
            ctx: *mut c::JSContext,
            _promise: c::JSValue,
            reason: c::JSValue,
            _is_handled: c_int,
            opaque: *mut c_void,
        ) {
            unsafe {
                let weak_self = &*(opaque as *const ServiceWeakRef);
                if let Some(service) = weak_self.upgrade() {
                    let c_reason = c::JS_ToCString(ctx, reason);
                    let mut exc_str = CStr::from_ptr(c_reason).to_string_lossy().into_owned();
                    let stack = c::JS_GetPropertyStr(ctx, reason, "stack\0".as_ptr() as _);
                    if !c::is_undefined(stack) {
                        exc_str.push_str("\n[stack]\n");
                        let c_lines = c::JS_ToCString(ctx, stack);
                        let lines = CStr::from_ptr(c_lines).to_string_lossy().into_owned();
                        exc_str.push_str(&lines);
                        c::JS_FreeCString(ctx, c_lines);
                    }
                    service
                        .unhandled_rejection_str
                        .borrow_mut()
                        .replace(exc_str);

                    c::JS_FreeValue(ctx, stack);
                    c::JS_FreeCString(ctx, c_reason);
                }
            }
        }

        unsafe {
            c::JS_SetHostPromiseRejectionTracker(
                runtime.as_ptr(),
                Some(promise_rejection_tracker),
                boxed_self as *mut _,
            )
        };

        if let Ok(v) = std::env::var("WAPO_RT_FLAGS") {
            if let Ok(v) = v.parse::<u32>() {
                runtime.set_debug_flags(v);
            }
        }
        let state = RefCell::new(ServiceState::default());
        {
            let mut abort_rx = runtime.subscribe_abort();
            let weak_self = weak_self.clone();
            crate::runtime::spawn(async move {
                _ = abort_rx.recv().await;
                debug!(target: "js::rt", "abort signal received");
                if let Some(service) = weak_self.upgrade() {
                    service.close_all();
                }
            });
        }

        Self {
            runtime: Rc::new_cyclic(|weak_self| JsEngine {
                runtime,
                ctx,
                weak_self: weak_self.clone(),
                last_error: Default::default(),
            }),
            state,
            config,
            unhandled_rejection_str: Default::default(),
        }
    }

    pub fn unhandled_rejection(&self) -> Result<Option<String>> {
        Ok(self.unhandled_rejection_str.borrow().clone())
    }

    pub fn boot(&self, bootcode: Option<&[u8]>) -> Result<()> {
        setup_host_functions(self.context(), &self.config)
            .context("failed to setup host functions")?;
        if let Some(bootcode) = bootcode {
            use js::NoStdContext;
            self.exec_bytecode(bootcode)
                .context("failed to execute boot code")?;
        }
        Ok(())
    }

    pub fn new_ref(config: ServiceConfig) -> ServiceRef {
        ServiceRef(Rc::new_cyclic(move |weak_self| {
            Service::new(ServiceWeakRef(weak_self.clone()), config)
        }))
    }

    pub fn worker_secret(&self) -> String {
        self.config.worker_secret.clone()
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

    pub fn run_default_module(&self) -> Result<js::Value> {
        let ctx = self.context();
        let default_fn = ctx
            .get_global_object()
            .get_property("Wapo")?
            .get_property("callModuleEntry")
            .unwrap_or_default();
        if default_fn.is_function() {
            return self.call_function(default_fn, ());
        }
        Ok(js::Value::Undefined)
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
            anyhow::bail!("failed to call function: {err}");
        }
        self.runtime.exec_pending_jobs();
        Ok(js::Value::new_moved(self.context(), ret))
    }

    pub fn push_resource(&self, resource: Resource) -> u64 {
        let mut state = self.state.borrow_mut();
        let id = state.take_next_resource_id();
        state.recources.insert(id, resource);
        debug!(target: "js::rt", "created resource {id}");
        id
    }

    pub fn get_resource_value(&self, id: u64) -> Option<js::Value> {
        let state = self.state.borrow();
        Some(self.to_js_value(&state.recources.get(&id)?.js_value))
    }

    pub fn close_all(&self) {
        debug!(target: "js::rt", "destroying all resources");
        let mut state = self.state.borrow_mut();
        if state.recources.is_empty() {
            return;
        }
        state.recources.clear();
        state.http_listener = None;
        state.query_listener = None;
        let _ = state.done_tx.send(());
    }

    pub fn remove_resource(&self, id: u64) -> Option<Resource> {
        debug!(target: "js::rt", "destroying resource {id}");
        let mut state = self.state.borrow_mut();
        let was_empty = state.is_empty();
        let res = state.recources.remove(&id);
        if !was_empty && state.is_empty() {
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
        self.spawn_with_cancel_rx(
            js_callback,
            move |srv, id, cancel_rx, args| async move {
                tokio::select! {
                    _ = fut_gen(srv, id, args) => {
                    }
                    _ = cancel_rx => {
                    }
                }
            },
            args,
        )
    }

    pub(crate) fn spawn_with_cancel_rx<Fut, FutGen, Args>(
        &self,
        js_callback: OwnedJsValue,
        fut_gen: FutGen,
        args: Args,
    ) -> u64
    where
        Fut: Future<Output = ()> + 'static,
        Args: 'static,
        FutGen: FnOnce(ServiceWeakRef, u64, oneshot::Receiver<()>, Args) -> Fut + 'static,
    {
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let res = Resource::new(js_callback, Some(Box::new(cancel_tx)));
        let id = self.push_resource(res);
        let weak_service = self.weak_self();
        let _handle = crate::runtime::spawn(async move {
            let _ = fut_gen(weak_service.clone(), id, cancel_rx, args).await;
            debug!(target: "js::rt", "task {id} finished");
            close(weak_service, id);
        });
        id
    }

    pub async fn wait_for_tasks(&self) {
        if self.state.borrow().is_empty() {
            return;
        }
        let mut rx = self.state.borrow().done_tx.subscribe();
        let _ = rx.recv().await;
    }

    pub fn number_of_tasks(&self) -> usize {
        self.state.borrow().recources.len()
    }

    pub fn set_http_listener(&self, listener: OwnedJsValue) {
        self.state.borrow_mut().http_listener = Some(listener);
    }

    pub fn set_query_listener(&self, listener: OwnedJsValue) {
        self.state.borrow_mut().query_listener = Some(listener);
    }

    pub fn http_listener(&self) -> Option<OwnedJsValue> {
        self.state.borrow().http_listener.as_ref()?.dup()
    }

    pub fn query_listener(&self) -> Option<OwnedJsValue> {
        self.state.borrow().query_listener.as_ref()?.dup()
    }

    pub fn to_js_value(&self, owned: &OwnedJsValue) -> js::Value {
        self.runtime.to_js_value(owned)
    }

    pub fn to_owned_value(&self, js_value: &js::Value) -> OwnedJsValue {
        self.runtime.to_owned_value(js_value)
    }

    /// Need to be called before dropping the service, this will drop all resources and
    /// wait for a while for their async tasks to finish so that the JS objects retained
    /// by the tasks are released before dropping the JS runtime.
    pub async fn shutdown(&self) {
        *self.state.borrow_mut() = Default::default();
        runtime::time::sleep(Duration::from_millis(2)).await;
    }
}

// Configuration get
impl Service {
    pub fn allow_isolate_eval(&self) -> bool {
        !self.config.is_sandbox
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
            if !self.state.borrow().recources.is_empty() {
                error!(target: "js::rt", "service dropped without explicit shutdown");
            }
            let pname = c::JS_GetContextOpaque(self.context().as_ptr()) as *mut ServiceWeakRef;
            drop(Box::from_raw(pname));
        }
    }
}
