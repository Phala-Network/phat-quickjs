use ink::env::hash::CryptoHash;

use qjsbind as js;

static mut CODE_PTR: *const core::ffi::c_void = core::ptr::null();
static mut CODE_LEN: usize = 0;

pub fn setup(pink: &js::Value) -> js::Result<()> {
    pink.define_property_fn("deriveSecret", derive_secret)?;
    Ok(())
}

/// Record the codes to be used to calculate the hash of the evaluating JS code.
///
/// Safety: Make sure the codes would live as long as the entire call of eval.
pub unsafe fn set_codes(codes: &[js::JsCode]) {
    if codes.len() == 0 {
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
        let codes = core::slice::from_raw_parts(CODE_PTR as *const js::JsCode<'static>, CODE_LEN);
        for code in codes {
            let hash = match code {
                js::JsCode::Source(src) => hash(&src.as_bytes()),
                js::JsCode::Bytecode(bytes) => hash(&bytes),
            };
            output.extend_from_slice(&hash);
        }
        return hash(&output);
    }
}

#[js::host_call]
fn derive_secret(salt: js::AsBytes<Vec<u8>>) -> js::AsBytes<Vec<u8>> {
    let mut seed = Vec::new();
    seed.extend_from_slice(b"JavaScript:");
    seed.extend_from_slice(&js_code_hash());
    seed.extend_from_slice(&salt.0);
    let secret = pink::ext().derive_sr25519_key(hash(&seed)[..].into());
    js::AsBytes(secret)
}
