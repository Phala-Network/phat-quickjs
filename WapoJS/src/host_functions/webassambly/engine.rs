use std::sync::{Mutex, Weak};

use anyhow::bail;

use super::externals::ExternObject;

pub type EngineStore = wasmi::Store<Data>;

environmental::environmental!(wasmi_using_context: EngineStore);

pub fn using_store<T>(store: &mut EngineStore, f: impl FnOnce() -> T) -> T {
    wasmi_using_context::using(store, f)
}

#[derive(Default)]
pub struct Data {
    ref_js_values: Mutex<Vec<Weak<js::Value>>>,
}

pub struct Store {
    store: EngineStore,
}

impl Data {
    pub fn push_ref(&self, value: Weak<js::Value>) {
        self.ref_js_values.lock().unwrap().push(value);
    }
}

impl AsRef<EngineStore> for Store {
    fn as_ref(&self) -> &EngineStore {
        &self.store
    }
}

impl AsMut<EngineStore> for Store {
    fn as_mut(&mut self) -> &mut EngineStore {
        &mut self.store
    }
}

impl js::GcMark for Store {
    fn gc_mark(&self, rt: *mut js::c::JSRuntime, mark_fn: js::c::JS_MarkFunc) {
        for buffer in self.store.iter_memory_buffers() {
            buffer.gc_mark(rt, mark_fn);
        }
        for ext in self.store.iter_extern_objects() {
            if let Some(ext) = ext.downcast_ref::<ExternObject>() {
                ext.gc_mark(rt, mark_fn);
            }
        }
        self.store
            .data()
            .ref_js_values
            .lock()
            .unwrap()
            .retain_mut(|value| {
                if let Some(value) = value.upgrade() {
                    value.gc_mark(rt, mark_fn);
                    true
                } else {
                    false
                }
            });
    }
}

#[derive(js::FromJsValue, js::GcMark, Clone)]
pub struct GlobalStore(js::Native<Store>);

impl From<GlobalStore> for js::Native<Store> {
    fn from(store: GlobalStore) -> js::Native<Store> {
        store.0
    }
}

js::impl_named!(Store as "WebAssembly.Store");

impl GlobalStore {
    pub fn engine(&self) -> wasmi::Engine {
        self.0.borrow().store.engine().clone()
    }

    fn try_borrow_mut(&self) -> js::Result<js::NativeValueRefMut<'_, Store>> {
        let r = self.0.borrow_mut();
        if r.is_none() {
            bail!("failed to borrow GlobalStore")
        }
        Ok(r)
    }

    pub fn with<T>(&self, f: impl FnOnce(&mut EngineStore) -> T) -> js::Result<T> {
        let rv = if wasmi_using_context::with(|_| ()).is_some() {
            wasmi_using_context::with(|ctx| f(ctx)).expect("should never fail")
        } else {
            let mut store = self.try_borrow_mut()?;
            f(store.as_mut())
        };
        Ok(rv)
    }
}

impl js::FromJsContext for GlobalStore {
    fn from_js_context(ctx: &js::Context) -> js::Result<Self> {
        let inner = ctx.get_qjsbind_object("wasm.global_store", || {
            let store = Store {
                store: wasmi::Store::default(),
            };
            js::Native::new_gc_obj_named(ctx, store)
        })?;
        Ok(js::FromJsValue::from_js_value(inner).expect("GlobalStore"))
    }
}
