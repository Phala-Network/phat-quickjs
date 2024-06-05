use super::*;

pub use bind::*;

pub fn setup(wasm_ns: &js::Value) -> Result<()> {
    use js::NativeClass;
    let constructor = Global::constructor_object(wasm_ns.context()?)?;
    wasm_ns.set_property("Global", &constructor)?;
    Ok(())
}

#[js::qjsbind]
mod bind {
    use anyhow::bail;
    use js::NoStdContext;

    use crate::host_functions::webassambly::{
        engine::GlobalStore,
        externals::{decode_type, decode_value_or_default, encode_value},
    };

    #[qjs(class(js_name = "WebAssembly.Global"))]
    pub struct Global {
        #[gc(skip)]
        val: wasmi::Global,
        store: GlobalStore,
    }

    #[derive(js::FromJsValue, Debug)]
    pub struct GlobalDescriptor {
        pub mutable: bool,
        pub value: String,
    }

    impl Global {
        #[qjs(constructor)]
        pub fn new(
            #[qjs(from_context)] store: GlobalStore,
            descriptor: GlobalDescriptor,
            value: js::Value,
        ) -> js::Result<Self> {
            let val = store.with(|store| -> js::Result<_> {
                let ty = decode_type(&descriptor.value)?;
                let initial_value = decode_value_or_default(store, ty, value)?;
                let mutability = if descriptor.mutable {
                    wasmi::Mutability::Var
                } else {
                    wasmi::Mutability::Const
                };
                Ok(wasmi::Global::new(store, initial_value, mutability))
            })??;
            Ok(Self { val, store })
        }

        #[qjs(setter, js_name = "value")]
        fn set_value(&self, val: js::Value) -> js::Result<()> {
            self.store.with(|store| -> js::Result<_> {
                let ty = self.val.ty(&*store);
                if ty.mutability().is_const() {
                    bail!("global is immutable");
                }
                let new_value = decode_value_or_default(store, ty.content(), val)?;
                self.val
                    .set(&mut *store, new_value)
                    .context("failed to set value")
            })??;
            Ok(())
        }

        #[qjs(getter)]
        fn value(&self, #[qjs(from_context)] ctx: js::Context) -> js::Result<js::Value> {
            self.store.with(|store| {
                let value = self.val.get(&*store);
                encode_value(store, &ctx, value)
            })?
        }

        #[qjs(method)]
        fn value_of(&self, #[qjs(from_context)] ctx: js::Context) -> js::Result<js::Value> {
            self.value(ctx)
        }

        pub fn raw_value(&self) -> &wasmi::Global {
            &self.val
        }

        pub fn from_raw(val: wasmi::Global, store: GlobalStore) -> Self {
            Self { val, store }
        }
    }
}
