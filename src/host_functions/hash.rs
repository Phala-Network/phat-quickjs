use anyhow::bail;
use js::AsBytes;
use sidevm::{env::HashAlgorithm, ocall};

use super::Result;

#[js::host_call]
fn hash(message: AsBytes<Vec<u8>>, algorithm: js::JsString) -> Result<AsBytes<Vec<u8>>> {
    let algo = match algorithm.as_str() {
        "twox64" => HashAlgorithm::Twox64,
        "twox128" => HashAlgorithm::Twox128,
        "twox256" => HashAlgorithm::Twox256,
        "sha2x256" => HashAlgorithm::Sha2x256,
        "keccak256" => HashAlgorithm::Keccak256,
        "keccak512" => HashAlgorithm::Keccak512,
        "blake2x64" => HashAlgorithm::Blake2x64,
        "blake2x128" => HashAlgorithm::Blake2x128,
        "blake2x256" => HashAlgorithm::Blake2x256,
        "blake2x512" => HashAlgorithm::Blake2x512,
        _ => bail!("Unsupported hash algorithm: {}", algorithm.as_str()),
    };
    Ok(ocall::hash(message.0.as_ref(), algo)?.into())
}

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("hash", hash)?;
    Ok(())
}
