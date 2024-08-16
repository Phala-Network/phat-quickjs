use anyhow::Result;
use crate::service::{OwnedJsValue, ServiceRef};
use sha3::{Digest, Sha3_512};
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
fn derive_secret_native(service: ServiceRef, _this: js::Value, message: js::BytesOrString, callback: OwnedJsValue) -> Result<js::AsBytes<[u8; 64]>> {
    match message.as_str() {
        Some(message) => {
            let secret = service.worker_secret();
            match secret {
                Some(secret) => {
                    let text = format!("{secret}::{message}");
                    let mut hasher = Sha3_512::new();
                    hasher.update(text.as_bytes());
                    Ok(js::AsBytes(hasher.finalize().into()))
                },
                None => {
                    Err(anyhow!("worker secret is not set"))
                }
            }
        },
        None => {
            Err(anyhow!(""))
        }

    }
}
