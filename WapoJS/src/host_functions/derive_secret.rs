use anyhow::Result;
use crate::service::ServiceRef;
use blake2::{Blake2b512, Digest};
use anyhow::anyhow;

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    #[cfg(feature = "wapo")]
    ns.define_property_fn("deriveSecret", derive_secret)?;
    #[cfg(feature = "native")]
    ns.define_property_fn("deriveSecret", derive_secret_native)?;
    #[cfg(all(not(feature = "wapo"), not(feature = "native")))]
    ns.define_property_fn("deriveSecret", qjs_extensions::sha3::sha3_512)?;
    Ok(())
}

#[cfg(feature = "wapo")]
#[js::host_call]
fn derive_secret(path: js::BytesOrString) -> Result<js::AsBytes<[u8; 64]>> {
    wapo::ocall::derive_secret(path.as_bytes())
        .map(js::AsBytes)
        .map_err(Into::into)
}

#[cfg(feature = "native")]
#[js::host_call(with_context)]
fn derive_secret_native(service: ServiceRef, _this: js::Value, message: js::BytesOrString) -> Result<js::AsBytes<[u8; 64]>> {
    let secret = service.worker_secret().ok_or(anyhow!("worker secret is not set"));
    let mut hasher = Blake2b512::new();
    hasher.update(secret?.as_bytes());
    hasher.update(message.as_bytes());
    Ok(js::AsBytes(hasher.finalize().into()))
}
