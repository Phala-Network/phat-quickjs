pub use bind::*;

pub fn setup(wasm_ns: &js::Value) -> js::Result<()> {
    use js::NativeClass;
    let constructor = Memory::constructor_object(wasm_ns.context()?)?;
    wasm_ns.set_property("Memory", &constructor)?;
    Ok(())
}

#[js::qjsbind]
mod bind {
    use js::NoStdContext;
    use wasmi::core::Pages;

    use crate::host_functions::webassambly::engine::GlobalStore;

    #[qjs(class(js_name = "WebAssembly.Memory"))]
    pub struct Memory {
        #[gc(skip)]
        memory: wasmi::Memory,
        store: GlobalStore,
    }

    #[derive(js::FromJsValue, Debug)]
    pub struct MemoryDescriptor {
        initial: u32,
        maximum: Option<u32>,
        #[qjs(default)]
        shared: bool,
    }

    impl Memory {
        #[qjs(constructor)]
        pub fn new(
            #[qjs(from_context)] js_ctx: js::Context,
            #[qjs(from_context)] store: GlobalStore,
            descriptor: MemoryDescriptor,
        ) -> js::Result<Self> {
            if descriptor.shared {
                return Err(js::Error::msg("shared memory is not supported"));
            }
            let memory = {
                let mem_ty = wasmi::MemoryType::new(descriptor.initial, descriptor.maximum)
                    .context("failed to create memory type")?;
                store.with(|store| {
                    wasmi::with_js_context(&js_ctx, || wasmi::Memory::new(store, mem_ty))
                        .context("failed to create memory")
                })??
            };
            Ok(Self { memory, store })
        }

        #[qjs(method)]
        pub fn grow(
            &self,
            #[qjs(from_context)] js_ctx: js::Context,
            delta: u32,
        ) -> js::Result<u32> {
            let additional_pages = Pages::new(delta).context("invalid number of pages")?;
            let prev_pages = self.store.with(|store| {
                wasmi::with_js_context(&js_ctx, || self.memory.grow(store, additional_pages))
                    .context("failed to grow memory")
            })??;
            Ok(prev_pages.into())
        }

        #[qjs(getter)]
        pub fn buffer(&self) -> js::Result<Option<js::JsArrayBuffer>> {
            self.store
                .with(|store| self.memory.js_buffer(store).cloned())
        }

        pub fn raw_value(&self) -> &wasmi::Memory {
            &self.memory
        }

        pub fn from_raw(memory: wasmi::Memory, store: GlobalStore) -> Self {
            Self { memory, store }
        }
    }
}
