use std::borrow::Cow;
use std::env;
use std::path::Path;

use js::ToJsValue;

use crate::{
    service::{ServiceConfig, ServiceRef},
    Service,
};
use anyhow::{anyhow, bail, Context, Result};

#[cfg(feature = "native")]
use dotenv;

use pink_types::js::{JsCode, JsValue};

struct Args {
    #[cfg(feature = "native")]
    tls_port: u16,
    codes: Vec<JsCode>,
    js_args: Vec<String>,
    worker_secret: String,
}

#[cfg(feature = "wapo")]
fn load_code(code_hash: &str) -> Result<String> {
    log::info!(target: "js", "loading code with hash: {code_hash}");
    let source_blob = wapo::ocall::blob_get(&code_hash).context("failed to get source code")?;
    let source_code = String::from_utf8(source_blob).context("source code is not valid utf-8")?;
    Ok(source_code)
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Args> {
    let mut codes = vec![];
    let mut iter = args.skip(1);
    #[cfg(feature = "native")]
    let mut tls_port = 443_u16;
    let mut worker_secret: Option<String> = None;
    while let Some(arg) = iter.next() {
        if arg.starts_with("-") {
            if arg == "--" {
                break;
            }
            match arg.as_str() {
                #[cfg(feature = "wapo")]
                "--code-hash" => {
                    let code_hash = iter
                        .next()
                        .ok_or(anyhow!("missing value after --code-hash"))?;
                    let code =
                        load_code(&code_hash).context("failed to load code with given hash")?;
                    codes.push(JsCode::Source(code));
                }
                #[cfg(feature = "native")]
                "--tls-port" => {
                    tls_port = iter
                        .next()
                        .ok_or(anyhow!("missing value after --tls-port"))?
                        .parse()?;
                }
                "-c" => {
                    let code = iter.next().ok_or(anyhow!("missing code after -c"))?;
                    codes.push(JsCode::Source(code));
                }
                #[cfg(feature = "native")]
                "-e" => {
                    let path_str = iter.next().ok_or(anyhow!("missing path after -e"))?;
                    let path = Path::new(&path_str);
                    if !path.exists() {
                        return Err(anyhow!("path {path_str} not exists."));
                    }
                    if let Err(_) = dotenv::from_path(path) {
                        return Err(anyhow!("not a valid env file: {path_str}"));
                    }
                }
                "--worker-secret" => {
                    let secret = iter
                        .next()
                        .ok_or(anyhow!("missing value after --worker-secret"))?;
                    worker_secret = Some(secret);
                }
                _ => {
                    print_usage();
                    bail!("unknown option: {}", arg);
                }
            }
        } else {
            // File name
            let code = std::fs::read_to_string(arg).context("failed to read script file")?;
            codes.push(JsCode::Source(code));
        }
    }
    if codes.is_empty() {
        print_usage();
        bail!("no script file provided");
    }
    if worker_secret.is_none() {
        print_usage();
        bail!("--worker-secret is required.");
    }
    let js_args = iter.collect();
    Ok(Args {
        codes,
        js_args,
        #[cfg(feature = "native")]
        tls_port,
        worker_secret: worker_secret.unwrap(),
    })
}

fn print_usage() {
    println!("wapojs v{}", env!("CARGO_PKG_VERSION"));
    println!("Usage: wapojs [options] --worker-secret <secret> [script..] [-- [args]]");
    println!("");
    println!("Options:");
    println!("  -c <code>        Execute code");
    #[cfg(feature = "wapo")]
    println!("  --code-hash <code_hash>  Execute code");
    #[cfg(feature = "native")]
    println!("  --tls-port <port>  TLS listen port (default: 443)");
    #[cfg(feature = "native")]
    println!("  -e <path>        dotenv file provides additional env variables");
    println!("  --worker-secret <secret>    Worker secret");
    println!("  --               Stop processing options");
}

pub async fn run(args: impl Iterator<Item = String>) -> Result<JsValue> {
    #[cfg(feature = "env-browser")]
    const DEFAULT_BOOTCODE: &[u8] = bootcode::BOOT_CODE_BROWSER;
    #[cfg(feature = "env-nodejs")]
    const DEFAULT_BOOTCODE: &[u8] = bootcode::BOOT_CODE_NODEJS;

    #[cfg(feature = "external-bootcode")]
    let bootcode: Cow<'_, [u8]> = if let Ok(bootcode_path) = std::env::var("WAPOJS_BOOTCODE") {
        let source = std::fs::read_to_string(bootcode_path).expect("failed to read bootcode");
        let code = js::compile(&source, "<bootcode>").expect("failed to compile bootcode");

        Cow::Owned(code)
    } else {
        Cow::Borrowed(DEFAULT_BOOTCODE)
    };
    #[cfg(not(feature = "external-bootcode"))]
    let bootcode = Cow::Borrowed(DEFAULT_BOOTCODE);

    let parsed_args = parse_args(args)?;

    let config = ServiceConfig {
        is_sandbox: false,
        engine_config: Default::default(),
        worker_secret: parsed_args.worker_secret.clone(),
    };

    let service = Service::new_ref(config);
    service.boot(Some(&bootcode))?;

    let rv = run_with_service(service.clone(), parsed_args).await;
    service.shutdown().await;
    rv
}

async fn run_with_service(
    service: ServiceRef,
    args: Args,
) -> Result<JsValue> {
    #[cfg(feature = "native")]
    {
        crate::runtime::set_sni_tls_port(args.tls_port);
    }
    let js_ctx = service.context();
    let js_args = args
        .js_args
        .to_js_value(&js_ctx)
        .context("failed to convert args to js value")?;
    js_ctx
        .get_global_object()
        .set_property("scriptArgs", &js_args)
        .context("failed to set scriptArgs")?;

    let mut expr_val = None;
    for code in args.codes.into_iter() {
        let result = match code {
            JsCode::Source(src) => service.exec_script(&src),
            JsCode::Bytecode(bytes) => service.exec_bytecode(&bytes),
        };
        match result {
            Ok(value) => expr_val = value.to_js_value(),
            Err(err) => {
                bail!("failed to execute script: {err}");
            }
        }
    }

    service.run_default_module()?;

    #[cfg(feature = "wapo")]
    loop {
        tokio::select! {
            _ = service.wait_for_tasks() => {
                break;
            }
            query = wapo::channel::incoming_queries().next() => {
                let Some(query) = query else {
                    log::info!(target: "js", "host dropped the channel, exiting...");
                    break;
                };
                crate::host_functions::try_accept_query(service.clone(), query)?;
            }
            request = wapo::channel::incoming_http_requests().next() => {
                let Some(request) = request else {
                    log::info!(target: "js", "host dropped the channel, exiting...");
                    break;
                };
                #[cfg(feature = "js-http-listen")]
                crate::host_functions::try_accept_http_request(service.clone(), request)?;
            }
        }
    }
    #[cfg(feature = "native")]
    {
        service.wait_for_tasks().await;
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
    convert(output).context("failed to convert output")
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
