#![cfg_attr(not(feature = "std"), no_std, no_main)]
extern crate alloc;

pub use play::*;

#[ink::contract]
mod play {
    use alloc::string::String;
    use alloc::vec::Vec;
    use phat_js::JsValue;

    #[ink(storage)]
    pub struct Play {}

    impl Play {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn eval_js(
            &self,
            driver: String,
            js: String,
            args: Vec<String>,
        ) -> Result<JsValue, String> {
            if driver == "AsyncJsRuntime" {
                Ok(phat_js::eval_async_js(&js, &args))
            } else {
                let driver = get_driver(driver)?;
                phat_js::eval_with(driver, &js, &args).map(Into::into)
            }
        }
    }

    fn get_driver(driver: String) -> Result<Hash, String> {
        let system = pink::system::SystemRef::instance();
        let delegate = system.get_driver(driver).ok_or("No JS driver found")?;
        Ok(phat_js::ConvertTo::convert_to(&delegate))
    }
}
