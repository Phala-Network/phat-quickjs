use super::Result;
use wyhash_final4::generics::WyHashVariant;
use wyhash_final4::wyhash64::*;
use js::AsBytes;

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("non_cryptographic_hash", non_cryptographic_hash)?;
    Ok(())
}

#[js::host_call]
fn non_cryptographic_hash(message: js::BytesOrString) -> Result<AsBytes<Vec<u8>>> {
    let hash = Vec::from(WyHash64::with_seed(0).hash(message.as_ref()).to_le_bytes());
    Ok(hash.into())
}
