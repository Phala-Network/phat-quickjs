use super::Result;
use anyhow::bail;
use js::AsBytes;

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("hash", hash_impl::hash)?;
    Ok(())
}

#[cfg(not(feature = "native"))]
mod hash_impl {
    use super::*;

    use sidevm::{env::HashAlgorithm, ocall};
    #[js::host_call]
    fn hash(message: AsBytes<Vec<u8>>, algorithm: js::JsString) -> Result<AsBytes<Vec<u8>>> {
        let algo = match algorithm.as_str() {
            "sha256" => HashAlgorithm::Sha2x256,
            "keccak256" => HashAlgorithm::Keccak256,
            "blake2b128" => HashAlgorithm::Blake2x128,
            "blake2b256" => HashAlgorithm::Blake2x256,
            _ => bail!("Unsupported hash algorithm: {}", algorithm.as_str()),
        };
        Ok(ocall::hash(message.0.as_ref(), algo)?.into())
    }
}

#[cfg(feature = "native")]
mod hash_impl {
    use super::*;
    use blake2::{
        digest::typenum::{U16, U32},
        Blake2b, Digest,
    };

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
}
