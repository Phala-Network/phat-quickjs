use alloc::string::String;
use alloc::vec::Vec;

use ink::env::hash::CryptoHash;
use qjsbind as js;

pub fn setup(pink: &js::Value) -> js::Result<()> {
    pink.define_property_fn("hash", hash)?;
    Ok(())
}

#[js::host_call]
fn hash(algorithm: String, message: js::AsBytes<Vec<u8>>) -> Result<js::AsBytes<Vec<u8>>, String> {
    let message = message.0;
    let hash = match algorithm.as_str() {
        "blake2b128" => {
            let mut output = Default::default();
            ink::env::hash::Blake2x128::hash(&message, &mut output);
            output.to_vec()
        }
        "blake2b256" => {
            let mut output = Default::default();
            ink::env::hash::Blake2x256::hash(&message, &mut output);
            output.to_vec()
        }
        "keccak256" => {
            let mut output = Default::default();
            ink::env::hash::Keccak256::hash(&message, &mut output);
            output.to_vec()
        }
        "sha256" => {
            let mut output = Default::default();
            ink::env::hash::Sha2x256::hash(&message, &mut output);
            output.to_vec()
        }
        _ => return Err(alloc::format!("Unsupported hash algorithm: {}", algorithm)),
    };
    Ok(js::AsBytes(hash))
}
