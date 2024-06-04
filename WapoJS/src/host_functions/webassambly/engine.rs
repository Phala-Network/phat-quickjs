use std::ops::{Deref, DerefMut};

type Data = ();

pub struct Store {
    store: wasmi::Store<Data>,
}

impl js::GcMark for Store {
    fn gc_mark(&self, rt: *mut js::c::JSRuntime, mark_fn: js::c::JS_MarkFunc) {
        for buffer in self.store.iter_memory_buffers() {
            buffer.gc_mark(rt, mark_fn);
        }
    }
}

impl Deref for Store {
    type Target = wasmi::Store<Data>;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl DerefMut for Store {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.store
    }
}

#[derive(js::FromJsValue)]
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

    pub fn borrow(&self) -> js::NativeValueRef<'_, Store> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> js::NativeValueRefMut<'_, Store> {
        self.0.borrow_mut()
    }

    pub fn into_inner(self) -> js::Native<Store> {
        self.0
    }

    pub fn inner(&self) -> &js::Native<Store> {
        &self.0
    }
}

impl js::FromJsContext for GlobalStore {
    fn from_js_context(ctx: &js::Context) -> js::Result<Self> {
        let inner = ctx.get_qjsbind_object("wasm.globalStore", || {
            let store = Store {
                store: wasmi::Store::default(),
            };
            js::Native::new_gc_obj_named(ctx, store)
        })?;
        Ok(js::FromJsValue::from_js_value(inner).expect("GlobalStore"))
    }
}
