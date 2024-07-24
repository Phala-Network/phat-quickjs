use anyhow::Result;
use js::AsBytes;

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("workerSign", worker_sign)?;
    ns.define_property_fn("workerPublicKey", worker_pubkey)?;
    ns.define_property_fn("sgxQuote", sgx_quote)?;
    ns.define_property_fn("bootData", boot_data)?;
    ns.define_property_fn("setBootData", set_boot_data)?;
    ns.define_property_fn("tryLock", try_lock)?;
    ns.define_property_fn("unlock", unlock)?;
    Ok(())
}

#[js::host_call]
fn worker_sign(message: js::BytesOrString) -> Result<AsBytes<Vec<u8>>> {
    wapo::ocall::sign(message.as_bytes())
        .map(AsBytes)
        .map_err(Into::into)
}

#[js::host_call]
fn worker_pubkey() -> Result<AsBytes<[u8; 32]>> {
    wapo::ocall::worker_pubkey()
        .map(AsBytes)
        .map_err(Into::into)
}

#[js::host_call]
fn sgx_quote(message: js::BytesOrString) -> Result<Option<AsBytes<Vec<u8>>>> {
    let quote = wapo::ocall::sgx_quote(message.as_bytes())?;
    Ok(quote.map(AsBytes))
}

#[js::host_call]
fn boot_data() -> Result<Option<js::Bytes>> {
    Ok(wapo::ocall::read_boot_data()?.map(Into::into))
}

#[js::host_call]
fn set_boot_data(data: js::Bytes) -> Result<()> {
    wapo::ocall::write_boot_data(data.as_bytes())?;
    Ok(())
}

struct Guard {
    path: String,
}

impl Drop for Guard {
    fn drop(&mut self) {
        let _ = wapo::ocall::app_unlock(&self.path);
    }
}

#[js::host_call(with_context)]
fn try_lock(context: js::Context, _this: js::Value, path: js::JsString) -> Result<js::Value> {
    wapo::ocall::app_try_lock(path.as_str())?;
    let gaurd = Guard {
        path: path.as_str().into(),
    };
    Ok(js::Value::new_opaque_object(
        &context,
        Some("AppLockGuard"),
        gaurd,
    ))
}

#[js::host_call]
fn unlock(guard: js::Value) -> Result<()> {
    let _guard = guard
        .opaque_object_take_data::<Guard>()
        .ok_or_else(|| anyhow::anyhow!("invalid lock guard"))?;
    Ok(())
}
