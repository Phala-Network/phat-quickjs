pub use bind::*;

pub fn setup(wasm_ns: &js::Value) -> js::Result<()> {
    use js::NativeClass;
    let constructor = Module::constructor_object(wasm_ns.context()?)?;
    wasm_ns.set_property("Module", &constructor)?;
    Ok(())
}

#[js::qjsbind]
mod bind {
    use anyhow::{bail, Context};
    use js::{Native, Result};
    use log::debug;
    use wasmi::ExternType;

    use crate::host_functions::webassambly::engine::GlobalStore;

    #[qjs(class)]
    pub struct Module {
        #[gc(skip)]
        pub(crate) module: wasmi::Module,
    }

    #[derive(js::ToJsValue)]
    pub struct ExportItem {
        name: String,
        kind: String,
    }

    #[derive(js::ToJsValue)]
    struct ImportItem {
        module: String,
        name: String,
        kind: String,
    }

    impl Module {
        #[qjs(constructor)]
        pub fn new(#[qjs(from_context)] store: GlobalStore, code: js::Bytes) -> Result<Self> {
            debug!(target: "js::wasm", "creating WASM module, code_length={}", code.len());
            let module = wasmi::Module::new(&store.engine(), &mut code.as_bytes())
                .context("failed to parse module")?;
            Ok(Self { module })
        }

        #[qjs(method)]
        fn custom_sections(
            _module: Native<Module>,
            _section_name: js::JsString,
        ) -> Result<Vec<js::Bytes>> {
            bail!("Module.customSections not implemented")
        }

        #[qjs(method)]
        fn exports(module: Native<Module>) -> Vec<ExportItem> {
            module
                .borrow()
                .module
                .exports()
                .map(|entry| ExportItem {
                    name: entry.name().to_string(),
                    kind: extern_type_kind(&entry.ty()).to_string(),
                })
                .collect()
        }

        #[qjs(method)]
        fn imports(module: Native<Module>) -> Vec<ImportItem> {
            module
                .borrow()
                .module
                .imports()
                .map(|entry| ImportItem {
                    module: entry.module().to_string(),
                    name: entry.name().to_string(),
                    kind: extern_type_kind(entry.ty()).to_string(),
                })
                .collect()
        }
    }

    pub fn extern_type_kind(ty: &ExternType) -> &'static str {
        match ty {
            ExternType::Global(_) => "global",
            ExternType::Table(_) => "table",
            ExternType::Memory(_) => "memory",
            ExternType::Func(_) => "function",
        }
    }
}
