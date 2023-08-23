use super::*;
use crate::{runtime::time::sleep, service::OwnedJsValue};
use qjs::{host_call, Value as JsValue};

pub(crate) fn setup(ns: &JsValue) -> Result<()> {
    ns.set_property_fn("setTimeout", set_timeout)?;
    ns.set_property_fn("setInterval", set_interval)?;
    Ok(())
}

#[host_call]
fn set_timeout(
    service: ServiceRef,
    _this: JsValue,
    callback: OwnedJsValue,
    timeout_ms: u64,
) -> Result<i32> {
    Ok(service.spawn(callback, do_set_timeout, timeout_ms.max(4)) as _)
}

#[host_call]
fn set_interval(
    service: ServiceRef,
    _this: JsValue,
    callback: OwnedJsValue,
    timeout_ms: u64,
) -> Result<i32> {
    Ok(service.spawn(callback, do_set_interval, timeout_ms.max(4)) as _)
}

fn try_fire_timer(service: &Weak<Service>, id: u64) -> Result<()> {
    let Some(service) = service.upgrade() else {
        anyhow::bail!("Timer {id} exited because the service has been dropped");
    };
    let Some(callback) = service.get_resource_value(id) else {
        anyhow::bail!("Timer {id} exited because the resource has been dropped");
    };
    if let Err(err) = service.call_function(callback.try_into()?, ()) {
        error!("Failed to fire timer {id}: {err}");
    }
    Ok(())
}

async fn do_set_timeout(service: ServiceWeakRef, id: u64, timeout_ms: u64) {
    sleep(std::time::Duration::from_millis(timeout_ms)).await;
    try_fire_timer(&service, id).ignore();
}

async fn do_set_interval(service: ServiceWeakRef, id: u64, timeout_ms: u64) {
    loop {
        sleep(std::time::Duration::from_millis(timeout_ms)).await;
        if try_fire_timer(&service, id).log_err().is_err() {
            break;
        }
    }
}
