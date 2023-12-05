#![cfg_attr(not(feature = "native"), no_main)]

extern crate alloc;

use js::ToJsValue;

use anyhow::{anyhow, bail, Context, Result};
use sidevm_quickjs::{runtime, Service};

use pink_types::js::{JsCode, JsValue};

struct Args {
    codes: Vec<JsCode>,
    js_args: Vec<String>,
}

fn parse_args() -> Result<Args> {
    let mut codes = vec![];
    let mut iter = std::env::args();
    iter.next();
    while let Some(arg) = iter.next() {
        if arg.starts_with("-") {
            if arg == "--" {
                break;
            }
            match arg.as_str() {
                "-c" => {
                    let code = iter.next().ok_or(anyhow!("Missing code after -c"))?;
                    codes.push(JsCode::Source(code));
                }
                "-b" => {
                    let code = iter.next().ok_or(anyhow!("Missing code after -b"))?;
                    let bytecode = hex::decode(code).context("Failed to decode bytecode")?;
                    codes.push(JsCode::Bytecode(bytecode));
                }
                _ => {
                    print_usage();
                    bail!("Unknown option: {}", arg);
                }
            }
        } else {
            // File name
            let code = std::fs::read_to_string(arg).context("Failed to read script file")?;
            codes.push(JsCode::Source(code));
        }
    }
    if codes.is_empty() {
        print_usage();
        bail!("No script file provided");
    }
    let js_args = iter.collect();
    Ok(Args { codes, js_args })
}

fn print_usage() {
    println!("Usage: phatjs [options] [script..] [-- [args]]");
    println!("");
    println!("Options:");
    println!("  -c <code>        Execute code");
    println!("  -b <hexed code>  Execute bytecode");
    println!("  --               Stop processing options");
}

async fn run() -> Result<JsValue> {
    let args = parse_args()?;
    let service = Service::new_ref();
    let js_ctx = service.context();
    let js_args = args
        .js_args
        .to_js_value(&js_ctx)
        .context("Failed to convert args to js value")?;
    js_ctx
        .get_global_object()
        .set_property("scriptArgs", &js_args)
        .context("Failed to set scriptArgs")?;
    let mut expr_val = None;
    for code in args.codes.into_iter() {
        let result = match code {
            JsCode::Source(src) => service.exec_script(&src),
            JsCode::Bytecode(bytes) => service.exec_bytecode(&bytes),
        };
        match result {
            Ok(value) => expr_val = value.to_js_value(),
            Err(err) => {
                bail!("Failed to execute script: {err}");
            }
        }
    }
    if service.number_of_tasks() > 0 {
        service.wait_for_tasks().await;
    }
    if let Some(error) = service.runtime().take_last_error() {
        return Err(anyhow!("{error}"));
    }
    // If scriptOutput is set, use it as output. Otherwise, use the last expression value.
    let output = js_ctx
        .get_global_object()
        .get_property("scriptOutput")
        .unwrap_or_default();
    let output = if output.is_undefined() {
        expr_val.unwrap_or_default()
    } else {
        output
    };
    convert(output).context("Failed to convert output")
}

fn convert(output: js::Value) -> Result<JsValue> {
    if output.is_undefined() {
        return Ok(JsValue::Undefined);
    }
    if output.is_null() {
        return Ok(JsValue::Null);
    }
    if output.is_string() {
        return Ok(JsValue::String(output.decode_string()?));
    }
    if output.is_uint8_array() {
        return Ok(JsValue::Bytes(output.decode_bytes()?));
    }
    return Ok(JsValue::Other(output.to_string()));
}

#[runtime::main]
async fn main() {
    runtime::init_logger();
    runtime::run_local(async {
        let output = match run().await {
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

#[cfg(not(feature = "native"))]
#[no_mangle]
extern "C" fn __main_argc_argv(_argc: i32, _argv: *const *const u8) -> i32 {
    0
}
