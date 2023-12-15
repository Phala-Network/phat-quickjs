use alloc::vec::Vec;
use ink::env::hash::CryptoHash;

use qjsbind as js;

static mut CODE_PTR: *const core::ffi::c_void = core::ptr::null();
static mut CODE_LEN: usize = 0;

pub fn setup(pink: &js::Value) -> js::Result<()> {
    pink.define_property_fn("deriveSecret", derive_secret)?;
    pink.define_property_fn("jsCodeHash", get_js_code_hash)?;
    Ok(())
}

/// Record the codes to be used to calculate the hash of the evaluating JS code.
///
/// Safety: Make sure the codes would live as long as the entire call of eval.
pub unsafe fn set_codes(codes: &[js::Code]) {
    if codes.is_empty() {
        return;
    }
    unsafe {
        CODE_PTR = (&codes[0]) as *const _ as _;
        CODE_LEN = codes.len();
    }
}

fn hash(message: &[u8]) -> [u8; 32] {
    let mut output = Default::default();
    ink::env::hash::Blake2x256::hash(message, &mut output);
    output
}

fn js_code_hash() -> [u8; 32] {
    unsafe {
        if CODE_PTR.is_null() {
            return Default::default();
        }
        let mut output = Vec::new();
        let codes = core::slice::from_raw_parts(CODE_PTR as *const js::Code<'static>, CODE_LEN);
        for code in codes {
            let hash = match code {
                js::Code::Source(src) => hash(src.as_bytes()),
                js::Code::Bytecode(bytes) => hash(bytes),
            };
            output.extend_from_slice(&hash);
        }
        hash(&output)
    }
}

/// Derive a secret key from the given salt.
///
/// The key would be derived with the formula: `derive_sr25519_key("JavaScript:" + js_code_hash + salt)`,
/// where `js_code_hash` is the hash of the JS code being evaluated. The js hash is calculated by:
/// js_code_hash = blake2x256(blake2x256(code1) + blake2x256(code2) + ... + blake2x256(codeN)).
#[js::host_call]
fn derive_secret(salt: js::AsBytes<Vec<u8>>) -> js::AsBytes<Vec<u8>> {
    let prefix = b"JavaScript:";
    let mut seed = Vec::with_capacity(prefix.len().saturating_add(32).saturating_add(salt.0.len()));
    seed.extend_from_slice(prefix);
    seed.extend_from_slice(&js_code_hash());
    seed.extend_from_slice(&salt.0);
    let secret = pink::ext().derive_sr25519_key(seed.into());
    js::AsBytes(secret)
}

#[js::host_call]
fn get_js_code_hash() -> js::AsBytes<[u8; 32]> {
    js_code_hash().into()
}
