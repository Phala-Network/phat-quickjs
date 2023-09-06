#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

pub use contract_qjs::*;

// mod contract_call;
mod host_functions;

#[ink::contract]
mod contract_qjs {
    use pink::info;

    use alloc::string::{String, ToString};
    use alloc::vec::Vec;
    use bootcode::BOOT_CODE;
    use qjsbind::{JsCode, ToJsValue as _};
    use scale::{Decode, Encode};

    use crate::host_functions::setup_host_functions;

    #[derive(Debug, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Output {
        String(String),
        Bytes(Vec<u8>),
        Undefined,
    }

    #[ink(storage)]
    pub struct QuickJS {}

    impl QuickJS {
        #[allow(clippy::should_implement_trait)]
        #[ink(constructor)]
        pub fn default() -> Self {
            QuickJS {}
        }

        #[ink(message)]
        pub fn eval(&self, js: String, args: Vec<String>) -> Result<Output, String> {
            info!("evaluating js, code len: {}", js.len());
            eval(JsCode::Source(&js), args)
        }

        #[ink(message)]
        pub fn eval_bytecode(
            &self,
            bytecode: Vec<u8>,
            args: Vec<String>,
        ) -> Result<Output, String> {
            info!("evaluating js bytecode, code len: {}", bytecode.len());
            eval(JsCode::Bytecode(&bytecode), args)
        }

        #[ink(message)]
        pub fn compile(&self, js: String) -> Result<Vec<u8>, String> {
            Ok(qjsbind::compile(&js, "<eval>")?)
        }
    }

    fn eval(code: JsCode, args: Vec<String>) -> Result<Output, String> {
        let rt = qjsbind::Runtime::new();
        let ctx = rt.new_context();

        setup_host_functions(&ctx)?;

        let args = args.to_js_value(ctx.ptr())?;
        let global = ctx.get_global_object();
        global.set_property("scriptArgs", &args)?;

        ctx.eval(&JsCode::Bytecode(BOOT_CODE))?;
        ctx.eval(&JsCode::Source(&set_version()))?;
        let output = ctx.eval(&code)?;
        if output.is_uint8_array() {
            let bytes = output.decode_bytes()?;
            return Ok(Output::Bytes(bytes));
        }
        if output.is_undefined() {
            return Ok(Output::Undefined);
        }
        Ok(Output::String(output.to_string()))
    }

    fn set_version() -> String {
        let version = env!("CARGO_PKG_VERSION");
        alloc::format!(
            r#"
            globalThis.pink.version = "{}";
            "#,
            version
        )
    }
}
