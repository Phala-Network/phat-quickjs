pub use bind::*;

pub fn setup(wasm_ns: &js::Value) -> js::Result<()> {
    use js::NativeClass;
    let constructor = Instance::constructor_object(wasm_ns.context()?)?;
    wasm_ns.set_property("Instance", &constructor)?;
    Ok(())
}

#[js::qjsbind]
mod bind {
    use std::{collections::BTreeMap, sync::Arc};

    use anyhow::bail;
    use js::{ErrorContext, FromJsValue, ToJsValue};
    use log::{debug, trace};
    use wasmi::{AsContextMut, ExternType, FuncType};

    use crate::host_functions::webassambly::{
        engine::{using_store, Data, GlobalStore},
        global::{decode_value_or_default, encode_value, Global},
        memory::Memory,
        module::Module,
    };

    #[qjs(class(js_name = "WebAssembly.Instance"))]
    pub struct Instance {
        #[gc(skip)]
        instance: Arc<wasmi::Instance>,
        store: GlobalStore,
    }

    struct JsFn {
        name: String,
        ty: FuncType,
        callback: Arc<js::Value>,
    }

    unsafe impl Send for JsFn {}
    unsafe impl Sync for JsFn {}

    impl JsFn {
        fn new(
            name: String,
            store: &mut wasmi::Store<Data>,
            ty: FuncType,
            callback: js::Value,
        ) -> Self {
            let callback = Arc::new(callback);
            let weak = Arc::downgrade(&callback);
            store.data().push_ref(weak);
            Self { name, ty, callback }
        }

        fn call(&self, args: &[wasmi::Val], outputs: &mut [wasmi::Val]) -> js::Result<()> {
            trace!(target: "js::wasm", "calling ext function: {}", self.name);
            if outputs.len() != self.ty.results().len() {
                bail!(
                    "expected {} results, got {}",
                    self.ty.results().len(),
                    outputs.len()
                );
            }
            let ctx = self.callback.context()?;
            let mut js_inputs = vec![];
            for arg in args {
                js_inputs.push(encode_value(&ctx, arg.clone())?);
            }
            let js_output = self.callback.call(&js::Value::undefined(), &js_inputs)?;
            let rm = log::trace!(target: "js::wasm", "js_output: {:?}", js_output);
            if self.ty.results().is_empty() {
                return Ok(());
            }
            if self.ty.results().len() == 1 {
                let val = decode_value_or_default(self.ty.results()[0], js_output)?;
                outputs[0] = val;
                return Ok(());
            }
            let js_outputs = <Vec<js::Value>>::from_js_value(js_output)?;
            for (i, val) in js_outputs.into_iter().enumerate() {
                outputs[i] = decode_value_or_default(self.ty.results()[i], val)?;
            }
            Ok(())
        }
    }

    #[qjs(class(js_name = "WebAssembly.HostFunction"))]
    struct HostFn {
        name: String,
        #[gc(skip)]
        ty: FuncType,
        #[gc(skip)]
        func: wasmi::Func,
    }

    impl HostFn {
        fn new(name: String, ty: FuncType, func: wasmi::Func) -> Self {
            Self { name, ty, func }
        }

        #[qjs(method)]
        fn call(
            &self,
            #[qjs(from_context)] ctx: js::Context,
            #[qjs(from_context)] store: GlobalStore,
            args: Vec<js::Value>,
        ) -> js::Result<js::Value> {
            let mut inputs = vec![];
            let mut outputs = vec![];
            let mut args_iter = args.into_iter();
            for ty in self.ty.params().iter() {
                let arg = args_iter.next().unwrap_or(js::Value::undefined());
                inputs.push(decode_value_or_default(*ty, arg)?);
            }
            for t in self.ty.results() {
                outputs.push(wasmi::Val::default(*t));
            }
            trace!(target: "js::wasm", "{} inputs : {:?}", self.name, inputs);
            store.with(|store| -> js::Result<_> {
                wasmi::with_js_context(&ctx, || -> js::Result<_> {
                    self.func
                        .call(store, &inputs, &mut outputs[..])
                        .context("failed to call host function")
                })
            })??;
            trace!(target: "js::wasm", "{} outputs: {:?}", self.name, outputs);
            let js_outputs = outputs
                .into_iter()
                .map(|val| encode_value(&ctx, val))
                .collect::<js::Result<Vec<_>>>()?;
            match js_outputs.len() {
                0 => Ok(js::Value::undefined()),
                1 => js_outputs[0].to_js_value(&ctx),
                _ => js_outputs.to_js_value(&ctx),
            }
        }
    }

    impl Instance {
        #[qjs(constructor)]
        pub fn new(
            #[qjs(from_context)] ctx: js::Context,
            #[qjs(from_context)] store: GlobalStore,
            module: js::Native<Module>,
            imports: js::Value,
        ) -> js::Result<Self> {
            let instance = store.with(|store| Self::new2(ctx, store, module, imports))??;
            Ok(Self {
                instance: Arc::new(instance),
                store,
            })
        }

        pub fn new2(
            ctx: js::Context,
            store: &mut wasmi::Store<Data>,
            module: js::Native<Module>,
            imports: js::Value,
        ) -> js::Result<wasmi::Instance> {
            debug!(target: "js::wasm", "creating WASM instance");
            let instance = {
                let module = module.borrow();
                let engine = store.engine().clone();
                let mut linker = wasmi::Linker::<Data>::new(&engine);
                for import in module.module.imports() {
                    let module_name = import.module();
                    let field_name = import.name();
                    let obj = imports
                        .get_property(module_name)?
                        .get_property(field_name)?;
                    debug!(target: "js::wasm", "importing {module_name}.{field_name}, type={:?}", import.ty());
                    match import.ty().clone() {
                        ExternType::Global(_) => {
                            let global = <js::Native<Global>>::from_js_value(obj)?;
                            let wasmi_global = global.borrow().raw_global().clone();
                            linker
                                .define(module_name, field_name, wasmi_global)
                                .context("failed to define global")?;
                        }
                        ExternType::Table(_) => todo!(),
                        ExternType::Memory(_) => {
                            let memory = <js::Native<Memory>>::from_js_value(obj)?;
                            let wasmi_memory = memory.borrow().raw_memory().clone();
                            linker
                                .define(module_name, field_name, wasmi_memory)
                                .context("failed to define memory")?;
                        }
                        ExternType::Func(ty) => {
                            if !obj.is_function() {
                                bail!("imported function {module_name}.{field_name} is not a function");
                            }
                            let name = format!("{module_name}.{field_name}");
                            let js_fn = JsFn::new(name, store, ty.clone(), obj);
                            linker.func_new(
                                module_name,
                                field_name,
                                ty,
                                move |mut caller, args, rets| {
                                    using_store(caller.as_context_mut().store, || {
                                        js_fn.call(args, rets).map_err(|e| {
                                            eprintln!("error calling imported function: {e}");
                                            wasmi::Error::new(e.to_string())
                                        })
                                    })
                                },
                            )?;
                        }
                    }
                }
                debug!(target: "js::wasm", "instantiating module");
                wasmi::with_js_context(&ctx, || -> js::Result<_> {
                    let instance = linker
                        .instantiate(&mut *store, &module.module)
                        .context("failed to instantiate module")?;
                    instance
                        .ensure_no_start(&mut *store)
                        .context("unexpected start function")
                })?
            };
            debug!(target: "js::wasm", "module instantiated");
            Ok(instance)
        }

        #[qjs(getter)]
        fn exports(
            &self,
            #[qjs(from_context)] ctx: js::Context,
        ) -> js::Result<BTreeMap<String, js::Value>> {
            let wrapper = ctx
                .get_qjsbind_object("wasm.host_fn_wrapper", || {
                    ctx.eval(&js::Code::Source(
                        r#"(function (fn) { return function(...args){return fn.call(args);}; })"#,
                    ))
                    .map_err(js::Error::msg)
                })
                .context("failed to create host function wrapper")?;
            self.store.with(|store| -> js::Result<_> {
                let mut output = BTreeMap::new();
                for entry in self.instance.exports(&*store) {
                    let name = entry.name().to_string();
                    match entry.ty(&*store) {
                        ExternType::Global(_) => {
                            let Some(global) = self.instance.get_global(&*store, &name) else {
                                continue;
                            };
                            let global = Global::from_raw(global, self.store.clone());
                            let js_global = ctx.wrap_native(global)?.to_js_value(&ctx)?;
                            output.insert(name, js_global);
                        }
                        ExternType::Table(todo) => {}
                        ExternType::Memory(_) => {
                            let Some(memory) = self.instance.get_memory(&*store, &name) else {
                                continue;
                            };
                            let memory = Memory::from_raw(memory, self.store.clone());
                            let js_memory = ctx.wrap_native(memory)?.to_js_value(&ctx)?;
                            output.insert(name, js_memory);
                        }
                        ExternType::Func(ty) => {
                            let f = entry.into_func().expect("must be a function");
                            let value = ctx
                                .wrap_native(HostFn::new(name.to_string(), ty, f))?
                                .to_js_value(&ctx)?;
                            let js_fn = wrapper.call(&js::Value::null(), &[value])?;
                            output.insert(name, js_fn);
                        }
                    }
                }
                Ok(output)
            })?
        }
    }
}
