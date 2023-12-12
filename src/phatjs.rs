#![cfg_attr(not(feature = "native"), no_main)]

extern crate alloc;
use sidevm_quickjs::{js_eval, runtime};

#[cfg(not(feature = "web"))]
#[runtime::main]
async fn main() {
    use pink_types::js::JsValue;
    runtime::init_logger();
    runtime::run_local(async {
        let output = match js_eval::run(std::env::args()).await {
            Ok(value) => value,
            Err(err) => JsValue::Exception(err.to_string()),
        };
        #[cfg(feature = "native")]
        log::info!("Script output: {:?}", output);
        #[cfg(not(feature = "native"))]
        sidevm::ocall::emit_program_output(&scale::Encode::encode(&output))
            .expect("Failed to emit program output");
    })
    .await;
}

#[cfg(feature = "web")]
mod web {
    use super::*;
    use pink_types::js::JsValue as QjsValue;
    use wasm_bindgen::JsValue as WebJsValue;

    #[wasm_bindgen::prelude::wasm_bindgen]
    pub async fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    #[wasm_bindgen::prelude::wasm_bindgen]
    pub async fn run(args: Vec<String>) -> Result<WebJsValue, WebJsValue> {
        runtime::init_logger();
        let result = js_eval::run(args.into_iter()).await;
        match result {
            Ok(value) => Ok({
                match value {
                    QjsValue::Undefined => WebJsValue::UNDEFINED,
                    QjsValue::Null => WebJsValue::NULL,
                    QjsValue::String(v) => v.into(),
                    QjsValue::Other(v) => v.into(),
                    QjsValue::Bytes(v) => js_sys::Uint8Array::from(v.as_slice()).into(),
                    QjsValue::Exception(err) => return Err(err.into()),
                }
            }),
            Err(err) => Err(err.to_string().into()),
        }
    }
}

#[cfg(not(feature = "native"))]
#[no_mangle]
extern "C" fn __main_argc_argv(_argc: i32, _argv: *const *const u8) -> i32 {
    0
}
