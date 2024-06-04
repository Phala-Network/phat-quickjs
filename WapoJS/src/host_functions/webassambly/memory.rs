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

    use crate::host_functions::webassambly::engine::{GlobalStore, Store};

    #[qjs(class(js_name = "WebAssembly.Memory"))]
    pub struct Memory {
        #[gc(skip)]
        memory: wasmi::Memory,
        store: js::Native<Store>,
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
            #[qjs(from_context)] wasm_store: GlobalStore,
            descriptor: MemoryDescriptor,
        ) -> js::Result<Self> {
            if descriptor.shared {
                return Err(js::Error::msg("shared memory is not supported"));
            }
            let memory = {
                let mut store = wasm_store.borrow_mut();
                let mem_ty = wasmi::MemoryType::new(descriptor.initial, descriptor.maximum)
                    .context("failed to create memory type")?;
                wasmi::with_js_context(&js_ctx, || wasmi::Memory::new(&mut **store, mem_ty))
                    .context("failed to create memory")?
            };
            let store = wasm_store.into_inner();
            Ok(Self { memory, store })
        }

        #[qjs(method)]
        pub fn grow(
            &self,
            #[qjs(from_context)] js_ctx: js::Context,
            delta: u32,
        ) -> js::Result<u32> {
            let mut store = self.store.borrow_mut();
            let additional_pages = Pages::new(delta).context("invalid number of pages")?;
            let prev_pages = wasmi::with_js_context(&js_ctx, || {
                self.memory.grow(&mut **store, additional_pages)
            })
            .context("failed to grow memory")?;
            Ok(prev_pages.into())
        }

        #[qjs(getter)]
        pub fn buffer(&self) -> Option<js::JsArrayBuffer> {
            self.memory.js_buffer(&**self.store.borrow_mut())
        }
    }
}
