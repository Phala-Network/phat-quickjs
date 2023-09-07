#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

pub use contract_qjs::*;

mod host_functions;

#[ink::contract]
mod contract_qjs {
    use pink::info;

    use alloc::string::{String, ToString};
    use alloc::vec::Vec;
    use bootcode::BOOT_CODE;
    use qjsbind::{JsCode, ToJsValue as _, Value as JsValue};

    use crate::host_functions::setup_host_functions;

    use phat_js::{Output, Value};

    #[ink(storage)]
    pub struct QuickJS {}

    impl QuickJS {
        #[allow(clippy::should_implement_trait)]
        #[ink(constructor)]
        pub fn default() -> Self {
            QuickJS {}
        }

        #[ink(message)]
        pub fn version(&self) -> this_crate::VersionTuple {
            this_crate::version_tuple!()
        }

        #[ink(message)]
        pub fn eval(&self, js: String, args: Vec<String>) -> Result<Output, String> {
            info!("evaluating js, code len: {}", js.len());
            eval(&[JsCode::Source(&js)], args)
        }

        #[ink(message)]
        pub fn eval_bytecode(
            &self,
            bytecode: Vec<u8>,
            args: Vec<String>,
        ) -> Result<Output, String> {
            info!("evaluating js bytecode, code len: {}", bytecode.len());
            eval(&[JsCode::Bytecode(&bytecode)], args)
        }

        #[ink(message)]
        pub fn eval_all(
            &self,
            codes: Vec<phat_js::Value>,
            args: Vec<String>,
        ) -> Result<Output, String> {
            info!("batch evaluating {} scripts", codes.len());
            let mut js_codes = Vec::new();
            for code in &codes {
                let js_code = match code {
                    Value::String(s) => {
                        info!("src len: {}", s.len());
                        JsCode::Source(s)
                    }
                    Value::Bytes(b) => {
                        info!("bytecode len: {}", b.len());
                        JsCode::Bytecode(b)
                    }
                    Value::Undefined => return Err("undefined code".to_string()),
                };
                js_codes.push(js_code);
            }
            let output = eval(&js_codes, args)?;
            Ok(output)
        }

        #[ink(message)]
        pub fn compile(&self, js: String) -> Result<Vec<u8>, String> {
            Ok(qjsbind::compile(&js, "<eval>")?)
        }
    }

    fn eval(codes: &[JsCode], args: Vec<String>) -> Result<Output, String> {
        let rt = qjsbind::Runtime::new();
        let ctx = rt.new_context();

        setup_host_functions(&ctx)?;

        let args = args.to_js_value(ctx.ptr())?;
        let global = ctx.get_global_object();
        global.set_property("scriptArgs", &args)?;

        ctx.eval(&JsCode::Bytecode(BOOT_CODE))?;
        ctx.eval(&JsCode::Source(&set_version()))?;
        let mut output = JsValue::undefined();
        for code in codes.iter() {
            output = ctx.eval(&code)?;
        }
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
