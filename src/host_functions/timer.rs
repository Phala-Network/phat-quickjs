use super::*;

pub(super) fn set_timeout(
    service: ServiceRef,
    ctx: *mut c::JSContext,
    args: &[c::JSValueConst],
) -> Result<JsValue> {
    let Some(callback) = args.get(0) else {
        anyhow::bail!("Invoking setTimeout without callback");
    };
    let timeout_ms: u64 = match args.get(1) {
        Some(timeout) => DecodeFromJSValue::decode(ctx, *timeout).map_err(Error::msg)?,
        None => anyhow::bail!("Invoking setTimeout without timeout"),
    };
    let callback = service.dup_value(*callback);
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let res = Resource::new(callback, Some(Box::new(tx)));
    let id = service.push_resource(res);
    let weak_service = ServiceRef::downgrade(&service);
    let _ = sidevm::spawn(async move {
        tokio::select! {
            _ = sidevm::time::sleep(std::time::Duration::from_millis(timeout_ms)) => {
                try_fire_timer(weak_service, id);
            }
            _ = rx => {
                debug!("Timer {id} canceled");
            }
        }
    });

    Ok(JsValue::Int(id as i32))
}

fn try_fire_timer(service: Weak<Service>, id: u64) {
    let Some(service) = service.upgrade() else {
        debug!("Timer {id} exited because the service is dropped");
        return;
    };
    let Some(res) = service.remove_resource(id) else {
        debug!("Timer {id} exited because the resource is dropped");
        return;
    };
    if let Err(err) = service.call_function(*res.js_value.value(), &[]) {
        error!("Failed to fire timer {id}: {err}");
    }
    debug!("Timer {id} fired");
}

pub(super) fn set_interval(
    service: ServiceRef,
    ctx: *mut c::JSContext,
    args: &[c::JSValueConst],
) -> Result<JsValue> {
    let Some(callback) = args.get(0) else {
        anyhow::bail!("Invoking setInterval without callback");
    };
    let timeout_ms: u64 = match args.get(1) {
        Some(timeout) => DecodeFromJSValue::decode(ctx, *timeout).map_err(Error::msg)?,
        None => anyhow::bail!("Invoking setInterval without timeout"),
    };
    let timeout_ms = timeout_ms.max(10);
    let callback = service.dup_value(*callback);
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let res = Resource::new(callback, Some(Box::new(tx)));
    let id = service.push_resource(res);
    let weak_service = ServiceRef::downgrade(&service);
    let _ = sidevm::spawn(async move {
        tokio::select! {
            _ = interval_loop(weak_service, timeout_ms, id) => {}
            _ = rx => {
                debug!("Timer {id} canceled");
            }
        }
    });

    Ok(JsValue::Int(id as i32))
}

async fn interval_loop(service: ServiceWeakRef, timeout_ms: u64, id: u64) {
    loop {
        sidevm::time::sleep(std::time::Duration::from_millis(timeout_ms)).await;
        try_fire_timer_interval(&service, id);
    }
}

fn try_fire_timer_interval(service: &Weak<Service>, id: u64) {
    let Some(service) = service.upgrade() else {
        debug!("Timer {id} exited because the service is dropped");
        return;
    };
    let Some(callback) = service.get_resource_value(id) else {
        debug!("Timer {id} exited because the resource is dropped");
        return;
    };
    if let Err(err) = service.call_function(*callback.value(), &[]) {
        error!("Failed to fire timer {id}: {err}");
    }
}
