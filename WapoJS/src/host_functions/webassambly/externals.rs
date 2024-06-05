use anyhow::bail;
use js::ToJsValue;
use wasmi::core::ValType;

use crate::host_functions::webassambly::engine::EngineStore;

use super::instance::WasmFn;

#[derive(Clone, js::GcMark)]
pub struct ExternObject(js::Value);
unsafe impl Send for ExternObject {}
unsafe impl Sync for ExternObject {}

impl From<js::Value> for ExternObject {
    fn from(value: js::Value) -> Self {
        Self(value)
    }
}

impl ExternObject {
    fn clone_inner(&self) -> js::Value {
        self.0.clone()
    }
}

pub fn encode_value(
    store: &EngineStore,
    ctx: &js::Context,
    value: wasmi::Val,
) -> js::Result<js::Value> {
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
        wasmi::Val::FuncRef(val) => {
            let Some(f) = val.func() else {
                return Ok(js::Value::null());
            };
            let ty = f.ty(store);
            WasmFn::new("<anonymous>".into(), ty, f.clone()).wrapped(ctx)
        }
        wasmi::Val::ExternRef(val) => {
            let Some(ext) = val.data(store) else {
                return Ok(js::Value::null());
            };
            let Some(ext) = ext.downcast_ref::<ExternObject>() else {
                bail!("invalid externref");
            };
            Ok(ext.clone_inner())
        }
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

pub fn decode_value_or_default(
    store: &mut EngineStore,
    ty: ValType,
    value: js::Value,
) -> js::Result<wasmi::Val> {
    Ok(decode_value(store, ty, value)?.unwrap_or(wasmi::Val::default(ty)))
}

pub fn decode_value(
    store: &mut EngineStore,
    ty: ValType,
    value: js::Value,
) -> js::Result<Option<wasmi::Val>> {
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
        ValType::FuncRef => {
            if value.is_null_or_undefined() {
                return Ok(None);
            }
            let funcref = value.get_property(WasmFn::flag_attr())?;
            let val = funcref.decode::<js::Native<WasmFn>>()?;
            let inner = val.borrow().func().clone();
            Some(wasmi::Val::FuncRef(inner.into()))
        }
        ValType::ExternRef => {
            if value.is_null_or_undefined() {
                return Ok(None);
            }
            let ext = wasmi::ExternRef::new::<ExternObject>(store, ExternObject::from(value));
            Some(wasmi::Val::ExternRef(ext))
        }
    };
    Ok(val)
}
