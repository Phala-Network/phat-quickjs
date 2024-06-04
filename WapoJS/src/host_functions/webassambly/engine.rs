use std::sync::Weak;

use anyhow::bail;

#[derive(Default)]
pub struct Data {
    ref_js_values: Vec<Weak<js::Value>>,
}

pub struct Store {
    store: wasmi::Store<Data>,
}

impl Store {
    pub fn push_ref(&mut self, value: Weak<js::Value>) {
        self.store.data_mut().ref_js_values.push(value);
    }
}

impl AsRef<wasmi::Store<Data>> for Store {
    fn as_ref(&self) -> &wasmi::Store<Data> {
        &self.store
    }
}

impl AsMut<wasmi::Store<Data>> for Store {
    fn as_mut(&mut self) -> &mut wasmi::Store<Data> {
        &mut self.store
    }
}

impl js::GcMark for Store {
    fn gc_mark(&mut self, rt: *mut js::c::JSRuntime, mark_fn: js::c::JS_MarkFunc) {
        for buffer in self.store.iter_memory_buffers() {
            buffer.gc_mark_ro(rt, mark_fn);
        }
        self.store.data_mut().ref_js_values.retain_mut(|value| {
            if let Some(value) = value.upgrade() {
                value.gc_mark_ro(rt, mark_fn);
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

    pub fn try_borrow(&self) -> js::Result<js::NativeValueRef<'_, Store>> {
        let r = self.0.borrow();
        if r.is_none() {
            bail!("failed to borrow GlobalStore")
        }
        Ok(r)
    }

    pub fn try_borrow_mut(&self) -> js::Result<js::NativeValueRefMut<'_, Store>> {
        let r = self.0.borrow_mut();
        if r.is_none() {
            bail!("failed to borrow GlobalStore")
        }
        Ok(r)
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
