use std::{collections::BTreeMap, rc::Weak, time::Duration};

use anyhow::{bail, Result};
use js::{EngineConfig, Error, ErrorContext, FromJsValue, ToJsValue};
use log::{error, info};
use tokio::sync::oneshot;
use wyhash_final4::generics::WyHashVariant;
use wyhash_final4::wyhash64::*;
use anyhow::anyhow;

use crate::{
    service::{OwnedJsValue, ServiceConfig, ServiceRef, ServiceWeakRef},
    Service,
};

#[derive(js::FromJsValue, Debug)]
#[qjs(rename_all = "camelCase")]
pub struct EvalArgs {
    scripts: Vec<js::String>,
    args: Vec<String>,
    env: BTreeMap<String, String>,

    gas_limit: Option<u32>,
    memory_limit: Option<u32>,
    time_limit: Option<u64>,
    polyfills: Vec<String>,
}

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("isolateEval", isolate_eval)?;
    Ok(())
}

#[js::host_call(with_context)]
fn isolate_eval(
    service: ServiceRef,
    _this: js::Value,
    args: EvalArgs,
    callback: OwnedJsValue,
) -> Result<u64> {
    if !service.allow_isolate_eval() {
        bail!("isolateEval is disabled");
    }
    if let Some(memory_limit) = args.memory_limit {
        if memory_limit < 1024 * 128 {
            bail!("memory limit is too low, at least 128KB is required");
        }
    }

    let scripts: Vec<u8> = args.scripts.iter().flat_map(|script| {
        script.to_string().into_bytes()
    }).collect();
    let code_hash = WyHash64::with_seed(0).hash(scripts.as_slice()).to_string();
    let mut inner_worker_secret: Option<String> = None;
    match service.worker_secret() {
        Some(secret) => {
            let formatted = format!("{secret}::{code_hash}");
            inner_worker_secret = Some(formatted);
        }
        None => {
            anyhow!("worker secret is not set");
        }
    }

    let config = ServiceConfig {
        engine_config: EngineConfig {
            gas_limit: args.gas_limit,
            memory_limit: args.memory_limit,
            time_limit: args.time_limit,
        },
        is_sandbox: true,
        worker_secret: inner_worker_secret,
    };
    let child_service = Service::new_ref(config);
    child_service
        .boot(None)
        .context("failed to boot child service")?;
    for polyfill in args.polyfills {
        match polyfill.as_str() {
            "browser" => {
                child_service
                    .exec_bytecode(bootcode::BOOT_CODE_BROWSER)
                    .map_err(Error::msg)?;
            }
            "nodejs" => {
                child_service
                    .exec_bytecode(bootcode::BOOT_CODE_NODEJS)
                    .map_err(Error::msg)?;
            }
            "wapo" => {
                child_service
                    .exec_bytecode(bootcode::BOOT_CODE_WAPO)
                    .map_err(Error::msg)?;
            }
            _ => {
                bail!("unknown polyfill: {}", polyfill);
            }
        }
    }
    let global_object = child_service.context().get_global_object();
    {
        let args = args
            .args
            .to_js_value(child_service.context())
            .context("failed to create args")?;
        global_object.set_property("scriptArgs", &args)?;
    }
    {
        let process = global_object.get_property("process")?;
        if process.is_object() {
            let env = args
                .env
                .to_js_value(child_service.context())
                .context("failed to create env")?;
            process.set_property("env", &env)?;
        }
    }

    let mut output = OwnedJsValue::Undefined;
    for script in args.scripts {
        output = child_service
            .exec_script(script.as_str())
            .map_err(Error::msg)?;
    }
    let output = output.to_js_value().unwrap_or(js::Value::Undefined);

    let default_fn = child_service.context().get_global_object().get_property("module")?.get_property("exports").unwrap_or_default();
    if default_fn.is_function() {
        let res = service.call_function(default_fn, ());
        match res {
            Ok(_) => {
            },
            Err(err) => {
                bail!("{err}");
            }
        }
    }

    let id = service.spawn_with_cancel_rx(
        callback,
        wait_child,
        (child_service, output, args.time_limit),
    );
    Ok(id)
}

async fn wait_child(
    service: ServiceWeakRef,
    res: u64,
    cancel_rx: oneshot::Receiver<()>,
    args: (ServiceRef, js::Value, Option<u64>),
) {
    let (child_service, output, timeout) = args;
    let timeout = timeout.unwrap_or(u64::MAX);
    tokio::select! {
        _ = cancel_rx => {
            child_service.close_all();
            info!(target: "js::isolate", "isolateEval stopped");
        }
        _ = crate::runtime::time::sleep(Duration::from_millis(timeout)) => {
            child_service.close_all();
        }
        _ = child_service.wait_for_tasks() => {}
    }
    child_service.shutdown().await;
    let output_obj = child_service
        .context()
        .get_global_object()
        .get_property("scriptOutput")
        .ok();

    let output = match output_obj {
        Some(output) if !output.is_undefined() => output,
        _ => output,
    };
    if output.is_null_or_undefined() {
        invoke_callback(&service, res, &output);
    } else if output.is_string() {
        invoke_callback(&service, res, &output.to_string());
    } else {
        match <Vec<u8>>::from_js_value(output) {
            Ok(bytes) => {
                invoke_callback(&service, res, &bytes);
            }
            Err(_) => {
                invoke_callback(&service, res, &"[object]".to_string());
            }
        }
    }
}

fn invoke_callback(weak_service: &Weak<Service>, id: u64, data: &dyn ToJsValue) {
    let Some(service) = weak_service.upgrade() else {
        info!(target: "js::isolate", "the service has been dropped");
        return;
    };
    let Some(callback) = service.get_resource_value(id) else {
        info!(target: "js::isolate", "the resource has been dropped");
        return;
    };
    if let Err(_) = service.call_function(callback, (data,)) {
        error!(target: "js::isolate", "[{id}] failed to report isolateEval output");
    }
}
