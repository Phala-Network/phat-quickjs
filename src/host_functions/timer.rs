use super::*;

pub(super) fn set_timeout(
    service: ServiceRef,
    ctx: *mut c::JSContext,
    args: &[c::JSValueConst],
    interval: bool,
) -> Result<JsValue> {
    let Some(callback) = args.get(0) else {
        anyhow::bail!("Invoking setTimeout without callback");
    };
    let timeout_ms: u64 = match args.get(1) {
        Some(timeout) => DecodeFromJSValue::decode(ctx, *timeout).anyhow()?,
        None => anyhow::bail!("Invoking setTimeout without timeout"),
    };
    // As specified in the HTML standard, browsers will enforce a minimum timeout of 4 milliseconds
    // once a nested call to setTimeout has been scheduled 5 times. Here we enforce to 4 all the time.
    let timeout_ms = timeout_ms.max(4);
    let callback = service.dup_value(*callback);
    let id = if interval {
        service.spawn(callback, do_set_interval, timeout_ms)
    } else {
        service.spawn(callback, do_set_timeout, timeout_ms)
    };
    Ok(JsValue::Int(id as i32))
}

fn try_fire_timer(service: &Weak<Service>, id: u64) -> Result<()> {
    let Some(service) = service.upgrade() else {
        anyhow::bail!("Timer {id} exited because the service is dropped");
    };
    let Some(callback) = service.get_resource_value(id) else {
        anyhow::bail!("Timer {id} exited because the resource is dropped");
    };
    if let Err(err) = service.call_function(*callback.value(), &[]) {
        error!("Failed to fire timer {id}: {err}");
    }
    Ok(())
}

async fn do_set_timeout(service: ServiceWeakRef, id: u64, timeout_ms: u64) {
    sidevm::time::sleep(std::time::Duration::from_millis(timeout_ms)).await;
    try_fire_timer(&service, id).ignore();
}

async fn do_set_interval(service: ServiceWeakRef, id: u64, timeout_ms: u64) {
    loop {
        sidevm::time::sleep(std::time::Duration::from_millis(timeout_ms)).await;
        if try_fire_timer(&service, id).log_err().is_err() {
            break;
        }
    }
}
