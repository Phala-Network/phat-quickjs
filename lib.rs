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
}

#[ink::contract]
mod qjs {
    use pink::info;
    use pink_extension as pink;

    use alloc::string::String;

    #[ink(storage)]
    pub struct JsTest {}

    impl JsTest {
        #[ink(constructor)]
        pub fn default() -> Self {
            JsTest {}
        }

        #[ink(message)]
        pub fn eval(&self, js: String) -> Result<String, String> {
            info!("evaluating js [{js}]");
            qjs_sys::eval(&js)
        }
    }
}
