#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use ink_lang as ink;

pub use qjs::*;

mod polyfill {
    use core::ffi::{c_int, c_uchar};
    use pink_extension::{error, info};

    #[no_mangle]
    extern "C" fn __pink_fd_write(fd: c_int, buf: *const c_uchar, len: usize) -> usize {
        // TODO: a more robust implementation.
        let bin = unsafe { core::slice::from_raw_parts(buf, len) };
        let message = core::str::from_utf8(bin)
            .unwrap_or("<Invalid UTF-8 string>")
            .trim_end();
        if message.is_empty() {
            return len;
        }
        match fd {
            1 => info!("JS: {}", message),
            2 => error!("JS: {}", message),
            _ => {}
        }
        len
    }

    #[no_mangle]
    extern "C" fn __pink_clock_time_get(_id: u32, _precision: u64, retptr0: *mut u64) -> u16 {
        let t = pink_extension::ext().untrusted_millis_since_unix_epoch() * 1_000_000;
        unsafe {
            *retptr0 = t;
        }
        0
    }
}

#[ink::contract]
mod qjs {
    use pink::info;
    use pink_extension as pink;

    use alloc::string::String;
    use alloc::vec::Vec;
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
            qjs_sys::eval(JsCode::Source(&js), &args).map(Into::into)
        }

        #[ink(message)]
        pub fn eval_bytecode(&self, js: Vec<u8>, args: Vec<String>) -> Result<Output, String> {
            info!("evaluating js bytecode, code len: {}", js.len());
            qjs_sys::eval(JsCode::Bytecode(&js), &args).map(Into::into)
        }
    }
}
