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
    use js::{NoStdContext, ToJsValue};
    use wasmi::core::ValType;

    use crate::host_functions::webassambly::engine::GlobalStore;

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

    struct Val(wasmi::Val);
    impl ToJsValue for Val {
        fn to_js_value(&self, ctx: &js::Context) -> js::Result<js::Value> {
            encode_value(ctx, self.0.clone())
        }
    }

    pub fn encode_value(ctx: &js::Context, value: wasmi::Val) -> js::Result<js::Value> {
        match value {
            wasmi::Val::I32(val) => val.to_js_value(ctx),
            wasmi::Val::I64(val) => val.to_js_value(ctx),
            wasmi::Val::F32(val) => {
                let fval = unsafe { core::mem::transmute::<u32, f32>(val.into()) };
                fval.to_js_value(ctx)
            }
            wasmi::Val::F64(val) => {
                let fval = unsafe { core::mem::transmute::<u64, f64>(val.into()) };
                fval.to_js_value(ctx)
            }
            _ => bail!("unimplmemented wasm primitive type"),
        }
    }

    pub fn decode_type(ty: &str) -> js::Result<ValType> {
        use ValType::*;
        let ty = match ty {
            "i32" => I32,
            "i64" => I64,
            "f32" => F32,
            "f64" => F64,
            "anyfunc" => FuncRef,
            "externref" => ExternRef,
            _ => bail!("invalid type"),
        };
        Ok(ty)
    }

    pub fn decode_value_or_default(ty: ValType, value: js::Value) -> js::Result<wasmi::Val> {
        Ok(decode_value(ty, value)?.unwrap_or(wasmi::Val::default(ty)))
    }

    pub fn decode_value(ty: ValType, value: js::Value) -> js::Result<Option<wasmi::Val>> {
        macro_rules! tr {
            () => {
                |v| unsafe { core::mem::transmute(v) }
            };
        }
        let val = match ty {
            ValType::I32 => value.decode::<Option<i32>>()?.map(wasmi::Val::I32),
            ValType::I64 => value.decode::<Option<i64>>()?.map(wasmi::Val::I64),
            ValType::F32 => value
                .decode::<Option<f32>>()?
                .map(tr!())
                .map(wasmi::Val::F32),
            ValType::F64 => value
                .decode::<Option<f64>>()?
                .map(tr!())
                .map(wasmi::Val::F64),
            ValType::FuncRef => bail!("decoding funcref is not implemented"),
            ValType::ExternRef => bail!("decoding externref is not implemented"),
        };
        Ok(val)
    }

    impl Global {
        #[qjs(constructor)]
        pub fn new(
            #[qjs(from_context)] store: GlobalStore,
            descriptor: GlobalDescriptor,
            value: js::Value,
        ) -> js::Result<Self> {
            let ty = decode_type(&descriptor.value)?;
            let initial_value = decode_value_or_default(ty, value)?;
            let mutability = if descriptor.mutable {
                wasmi::Mutability::Var
            } else {
                wasmi::Mutability::Const
            };
            let val = store.with(|store| wasmi::Global::new(store, initial_value, mutability))?;
            Ok(Self { val, store })
        }

        #[qjs(setter, js_name = "value")]
        fn set_value(&self, val: js::Value) -> js::Result<()> {
            self.store.with(|store| -> js::Result<_> {
                let ty = self.val.ty(&*store);
                if ty.mutability().is_const() {
                    bail!("global is immutable");
                }
                let new_value = decode_value_or_default(ty.content(), val)?;
                self.val
                    .set(&mut *store, new_value)
                    .context("failed to set value")
            })??;
            Ok(())
        }

        #[qjs(getter)]
        fn value(&self) -> js::Result<Val> {
            let val = self.store.with(|store| self.val.get(store))?;
            Ok(Val(val))
        }

        #[qjs(method)]
        fn value_of(&self) -> js::Result<Val> {
            self.value()
        }

        pub fn raw_global(&self) -> &wasmi::Global {
            &self.val
        }

        pub fn from_raw(val: wasmi::Global, store: GlobalStore) -> Self {
            Self { val, store }
        }
    }
}
