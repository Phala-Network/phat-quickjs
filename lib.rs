#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

pub use qjs::*;

mod contract_call;
mod host_functions;

static mut CODE_HASH: [u8; 32] = [0; 32];

fn code_hash() -> [u8; 32] {
    unsafe { CODE_HASH }
}
fn calc_and_set_code_hash(code: &qjs_sys::JsCode) {
    let mut hash = [0; 32];
    let code = match code {
        JsCode::Source(src) => src.to_bytes(),
        JsCode::Bytecode(code) => code,
    };
    ink::env::hash_bytes::<ink::env::hash::Sha2x256>(code, &mut hash);
    unsafe {
        CODE_HASH = hash;
    }
}

#[ink::contract]
mod qjs {
    use pink::info;
    use pink_extension as pink;

    use alloc::vec::Vec;
    use alloc::{ffi::CString, string::String};
    use bootcode::BOOT_CODE;
    pub use qjs_sys::JsCode;
    use scale::{Decode, Encode};

    #[derive(Debug, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Output {
        String(String),
        Bytes(Vec<u8>),
        Undefined,
    }

    impl From<qjs_sys::Output> for Output {
        fn from(output: qjs_sys::Output) -> Self {
            match output {
                qjs_sys::Output::String(s) => Output::String(s),
                qjs_sys::Output::Bytes(b) => Output::Bytes(b),
                qjs_sys::Output::Undefined => Output::Undefined,
            }
        }
    }

    #[ink(storage)]
    pub struct QuickJS {}

    impl QuickJS {
        #[ink(constructor)]
        pub fn default() -> Self {
            QuickJS {}
        }

        #[ink(message)]
        pub fn eval(&self, js: String, args: Vec<String>) -> Result<Output, String> {
            info!("evaluating js, code len: {}", js.len());
            let code = alloc::ffi::CString::new(js).or(Err("Invalid encoding"))?;
            eval(JsCode::Source(&code), args)
        }

        #[ink(message)]
        pub fn eval_bytecode(
            &self,
            bytecode: Vec<u8>,
            args: Vec<String>,
        ) -> Result<Output, String> {
            info!("evaluating js bytecode, code len: {}", bytecode.len());
            eval(JsCode::Bytecode(&bytecode), args)
        }
    }

    fn eval(code: JsCode, args: Vec<String>) -> Result<Output, String> {
        super::calc_and_set_code_hash(&code);
        qjs_sys::eval(
            &[
                JsCode::Bytecode(BOOT_CODE),
                JsCode::Source(&set_version()),
                code,
            ],
            &args,
        )
        .map(Into::into)
    }

    fn set_version() -> CString {
        let version = env!("CARGO_PKG_VERSION");
        CString::new(alloc::format!(
            r#"
            globalThis.pink.version = "{}";
            "#,
            version
        ))
        .unwrap()
    }
}
