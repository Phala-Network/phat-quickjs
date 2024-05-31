#[js::qjsbind]
mod wasm {
    use wasmi::Engine;

    use crate::host_functions::webassambly::default_engine;

    #[qjs(class)]
    struct Module {
        #[qjs(no_gc)]
        module: wasmi::Module,
    }

    impl Module {
        #[qjs(constructor)]
        pub fn new(code: js::Bytes) -> js::Result<Self> {
            let engine = default_engine();
            let module = wasmi::Module::new(&engine, &mut code.as_bytes())?;
            todo!()
        }
    }
}
