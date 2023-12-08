use super::Result;
use anyhow::bail;
use blake2::{
    digest::typenum::{U16, U32},
    Blake2b, Digest,
};
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
fn hash(algorithm: js::JsString, message: AsBytes<Vec<u8>>) -> Result<AsBytes<Vec<u8>>> {
    let hash = match algorithm.as_str() {
        "sha256" => do_hash::<sha2::Sha256>(message.0),
        "keccak256" => do_hash::<sha3::Keccak256>(message.0),
        "blake2b128" => do_hash::<Blake2b<U16>>(message.0),
        "blake2b256" => do_hash::<Blake2b<U32>>(message.0),
        _ => bail!("Unsupported hash algorithm: {}", algorithm.as_str()),
    };
    Ok(hash.into())
}
