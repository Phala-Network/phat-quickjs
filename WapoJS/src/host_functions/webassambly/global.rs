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

    use crate::host_functions::webassambly::engine::GlobalStore;

    use super::engine::Store;

    #[qjs(class(js_name = "WebAssembly.Global"))]
    pub struct Global {
        #[gc(skip)]
        val: wasmi::Global,
        store: js::Native<Store>,
    }

    #[derive(js::FromJsValue, Debug)]
    pub struct GlobalDescriptor {
        pub mutable: bool,
        pub value: String,
    }

    struct Val(wasmi::Val);
    impl js::ToJsValue for Val {
        fn to_js_value(&self, ctx: &js::Context) -> js::Result<js::Value> {
            match self.0 {
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
                _ => bail!("invalid type"),
            }
        }
    }

    fn decode_value(ty: &str, value: js::Value) -> js::Result<wasmi::Val> {
        let val = match ty {
            "i32" => wasmi::Val::I32(value.decode::<Option<i32>>()?.unwrap_or_default()),
            "i64" => wasmi::Val::I64(value.decode::<Option<i64>>()?.unwrap_or_default()),
            "f32" => wasmi::Val::F32(unsafe {
                core::mem::transmute::<f32, u32>(value.decode::<Option<f32>>()?.unwrap_or_default())
                    .into()
            }),
            "f64" => wasmi::Val::F64(unsafe {
                core::mem::transmute::<f64, u64>(value.decode::<Option<f64>>()?.unwrap_or_default())
                    .into()
            }),
            "v128" => {
                bail!("v128 not implemented")
            }
            "externref" => {
                bail!("externref not implemented")
            }
            "funcref" => {
                bail!("funcref not implemented")
            }
            _ => bail!("invalid type {ty}"),
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
            let initial_value = decode_value(&descriptor.value, value)?;
            let mutability = if descriptor.mutable {
                wasmi::Mutability::Var
            } else {
                wasmi::Mutability::Const
            };
            let val = wasmi::Global::new(&mut **store.borrow_mut(), initial_value, mutability);
            Ok(Self {
                val,
                store: store.into_inner(),
            })
        }

        #[qjs(setter, js_name = "value")]
        fn set_value(&self, val: js::Value) -> js::Result<()> {
            let ty = self.val.ty(&mut **self.store.borrow_mut());
            if ty.mutability().is_const() {
                bail!("global is immutable");
            }
            use wasmi::core::ValType::*;
            let new_value = match ty.content() {
                I32 => decode_value("i32", val)?,
                I64 => decode_value("i64", val)?,
                F32 => decode_value("f32", val)?,
                F64 => decode_value("f64", val)?,
                _ => bail!("unsupported global value type"),
            };
            self.val
                .set(&mut **self.store.borrow_mut(), new_value)
                .context("failed to set value")?;
            Ok(())
        }

        #[qjs(getter)]
        fn value(&self) -> js::Result<Val> {
            let val = self.val.get(&mut **self.store.borrow_mut());
            Ok(Val(val))
        }

        #[qjs(method)]
        fn value_of(&self) -> js::Result<Val> {
            self.value()
        }
    }
}
