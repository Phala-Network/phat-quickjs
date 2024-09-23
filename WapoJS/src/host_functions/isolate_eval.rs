use std::{collections::BTreeMap, rc::Weak, time::Duration};

use anyhow::{bail, Result};
use blake2::{Blake2b512, Digest};
use js::{EngineConfig, Error, ErrorContext, FromJsValue, ToJsValue};
use log::{error, info};
use tokio::sync::oneshot;

use crate::{
    service::{OwnedJsValue, ServiceConfig, ServiceRef, ServiceWeakRef},
    Service,
};

trait CrossContext {
    fn transfer(self) -> Transferring<Self>
    where
        Self: Sized;
}

impl CrossContext for js::Value {
    fn transfer(self) -> Transferring<Self> {
        Transferring(self)
    }
}

/// A wrapper that allows transferring a value between JS contexts safely.
struct Transferring<T>(T);

impl ToJsValue for Transferring<js::Value> {
    fn to_js_value(&self, context: &js::Context) -> Result<js::Value> {
        if self.0.is_undefined() {
            return Ok(js::Value::Undefined);
        }
        if self.0.is_null() {
            return Ok(js::Value::Null);
        }
        if let Ok(s) = js::JsString::from_js_value(self.0.clone()) {
            return s.as_str().to_js_value(context);
        }
        if let Ok(n) = i32::from_js_value(self.0.clone()) {
            return n.to_js_value(context);
        }
        if let Ok(b) = bool::from_js_value(self.0.clone()) {
            return b.to_js_value(context);
        }
        if let Ok(bytes) = js::JsUint8Array::from_js_value(self.0.clone()) {
            return js::AsBytes(bytes.as_bytes()).to_js_value(context);
        }
        if let Ok(buffer) = js::JsArrayBuffer::from_js_value(self.0.clone()) {
            let obj = js::JsArrayBuffer::new(context, buffer.len())?;
            obj.fill_with_bytes(buffer.as_bytes());
            return obj.to_js_value(context);
        }
        Ok(js::Value::Undefined)
    }
}

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

    let mut hasher = Blake2b512::new();
    args.scripts.iter().for_each(|script| {
        hasher.update(script.as_bytes());
    });
    let code_hash = hasher.finalize();
    let secret = service.worker_secret();
    let inner_worker_secret = format!("{secret}::{code_hash:02x}");

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
    let mut error = None;
    for script in args.scripts {
        let result = child_service
            .exec_script(script.as_str());
        if let Err(e) = result {
            error = Some(Error::msg(e.to_string()));
            break;
        } else {
            output = result.unwrap();
        }
    }

    if error.is_none() {
        child_service.run_default_module()?;
    }

    let output = output.to_js_value().unwrap_or(js::Value::Undefined);

    let id = service.spawn_with_cancel_rx(
        callback,
        wait_child,
        (child_service, output, args.time_limit),
    );

    if error.is_none() {
        return Ok(id);
    }
    return Err(error.unwrap());
}

async fn wait_child(
    service: ServiceWeakRef,
    res: u64,
    cancel_rx: oneshot::Receiver<()>,
    args: (ServiceRef, js::Value, Option<u64>),
) {
    let (child_service, fallback, timeout) = args;
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

    let global_object = child_service.context().get_global_object();

    // let output = get_script_output(&global_object, _output);
    let output_obj = global_object.get_property("scriptOutput").ok();
    let output_obj = match output_obj {
        Some(output) if !output.is_undefined() => output,
        _ => fallback,
    };
    let serialized_obj = global_object
        .get_property("serializedScriptOutput")
        .unwrap_or_default();
    let logs_obj = global_object.get_property("scriptLogs").unwrap_or_default();
    let unhandled_rejection = child_service.unhandled_rejection().unwrap_or_default();

    invoke_callback(
        &service,
        res,
        &(
            unhandled_rejection,
            output_obj.transfer(),
            serialized_obj.transfer(),
            logs_obj.transfer(),
        ),
    );
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
