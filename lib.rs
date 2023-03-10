#![cfg_attr(not(feature = "std"), no_std)]
#![feature(default_alloc_error_handler)]

extern crate alloc;

#[cfg(not(feature = "std"))]
#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

pub use qjs::*;

mod contract_call;
mod polyfill;

#[ink::contract]
mod qjs {
    use pink::info;
    use pink_extension as pink;

    use alloc::string::String;
    use alloc::vec::Vec;
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
            qjs_sys::eval(&[JsCode::Bytecode(BOOT_CODE), JsCode::Source(&code)], &args)
                .map(Into::into)
        }

        #[ink(message)]
        pub fn eval_bytecode(
            &self,
            bytecode: Vec<u8>,
            args: Vec<String>,
        ) -> Result<Output, String> {
            info!("evaluating js bytecode, code len: {}", bytecode.len());
            qjs_sys::eval(
                &[JsCode::Bytecode(BOOT_CODE), JsCode::Bytecode(&bytecode)],
                &args,
            )
            .map(Into::into)
        }
    }
}
