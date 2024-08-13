use super::Result;
use anyhow::bail;
use blake2::{
    digest::typenum::{U16, U32, U64},
    Blake2b, Digest,
};
use wyhash_final4::generics::WyHashVariant;
use wyhash_final4::wyhash64::*;
use js::AsBytes;

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("hash", hash)?;
    Ok(())
}

fn do_hash<T: Digest>(data: impl AsRef<[u8]>) -> Vec<u8> {
    let mut hasher = T::new();
    hasher.update(data.as_ref());
    hasher.finalize().to_vec()
}

#[js::host_call]
fn hash(algorithm: js::JsString, message: js::BytesOrString) -> Result<AsBytes<Vec<u8>>> {
    let hash = match algorithm.as_str() {
        "sha256" => do_hash::<sha2::Sha256>(message),
        "keccak256" => do_hash::<sha3::Keccak256>(message),
        "blake2b128" => do_hash::<Blake2b<U16>>(message),
        "blake2b256" => do_hash::<Blake2b<U32>>(message),
        "blake2b512" => do_hash::<Blake2b<U64>>(message),
        "wyhash64" => Vec::from(WyHash64::with_seed(0).hash(message.as_ref()).to_le_bytes()),
        _ => bail!("unsupported hash algorithm: {}", algorithm.as_str()),
    };
    Ok(hash.into())
}
