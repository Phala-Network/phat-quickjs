use super::*;

pub use bind::*;

pub fn setup(wasm_ns: &js::Value) -> Result<()> {
    use js::NativeClass;
    let constructor = Table::constructor_object(wasm_ns.context()?)?;
    wasm_ns.set_property("Table", &constructor)?;
    Ok(())
}

#[js::qjsbind]
mod bind {
    use anyhow::anyhow;

    use crate::host_functions::webassambly::{
        engine::GlobalStore,
        externals::{decode_type, decode_value_or_default, encode_value},
    };

    #[qjs(class(js_name = "WebAssembly.Table"))]
    pub struct Table {
        #[gc(skip)]
        val: wasmi::Table,
    }

    #[derive(js::FromJsValue, Debug)]
    struct TableDescriptor {
        element: String,
        initial: u32,
        maximum: Option<u32>,
    }

    impl Table {
        #[qjs(constructor)]
        fn new(
            #[qjs(from_context)] store: GlobalStore,
            descriptor: TableDescriptor,
            value: js::Value,
        ) -> js::Result<Self> {
            let val = store.with(|store| -> js::Result<_> {
                let ty = decode_type(&descriptor.element)?;
                let initial_value = decode_value_or_default(store, ty, value)?;
                let ty = wasmi::TableType::new(ty, descriptor.initial, descriptor.maximum);
                Ok(wasmi::Table::new(store, ty, initial_value).map_err(|e| anyhow!(e))?)
            })??;
            Ok(Self { val })
        }

        #[qjs(getter)]
        fn length(&self, #[qjs(from_context)] store: GlobalStore) -> js::Result<u32> {
            store.with(|store| -> u32 { self.val.size(store) })
        }

        #[qjs(method)]
        fn get(
            &self,
            index: u32,
            #[qjs(from_context)] store: GlobalStore,
            #[qjs(from_context)] ctx: js::Context,
        ) -> js::Result<js::Value> {
            store.with(|store| {
                let val = self.val.get(&*store, index).ok_or(anyhow!("RangeError"))?;
                encode_value(store, &ctx, val)
            })?
        }

        #[qjs(method)]
        pub fn set(
            &self,
            index: u32,
            value: js::Value,
            #[qjs(from_context)] store: GlobalStore,
        ) -> js::Result<()> {
            store.with(|store| -> js::Result<_> {
                let ty = self.val.ty(&*store);
                let new_value = decode_value_or_default(store, ty.element(), value)?;
                self.val
                    .set(&mut *store, index, new_value)
                    .map_err(|e| anyhow!(e))
            })?
        }

        #[qjs(method)]
        fn grow(
            &self,
            delta: u32,
            value: js::Value,
            #[qjs(from_context)] store: GlobalStore,
        ) -> js::Result<u32> {
            store.with(|store| {
                let ty = self.val.ty(&*store);
                let new_value = decode_value_or_default(store, ty.element(), value)?;
                self.val
                    .grow(&mut *store, delta, new_value)
                    .map_err(|e| anyhow!(e))
            })?
        }

        pub fn raw_value(&self) -> &wasmi::Table {
            &self.val
        }

        pub fn from_raw(val: wasmi::Table) -> Self {
            Self { val }
        }
    }
}
