#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[macro_use]
extern crate alloc;

pub use control::*;

#[ink::contract]
mod control {
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;

    #[ink(storage)]
    pub struct Control {}

    impl Control {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn start_sidevm(&self, code: [u8; 32]) {
            pink::start_sidevm(code).expect("start sidevm failed");
        }

        #[ink(message)]
        pub fn sidevm_init_script(&self) -> String {
            include_str!("../../../examples/httpListen.js").to_string()
        }

        #[ink(message)]
        pub fn query_sidevm(&self, payload: String) -> String {
            let url = format!("sidevm://{}", hex::encode(self.env().account_id()));
            let payload = payload.as_bytes().to_vec();
            let response = pink::http_post!(url, payload);
            if response.status_code != 200 {
                return format!(
                    "SideVM query failed: {} {}: {}",
                    response.status_code,
                    response.reason_phrase,
                    String::from_utf8_lossy(&response.body)
                );
            }
            String::from_utf8_lossy(&response.body).to_string()
        }

        #[ink(message)]
        pub fn run_js_using_delegate2(&self, script: String, args: Vec<String>) -> Result<(), String> {
            let delegate = get_driver("JsDelegate2")?;
            phat_js::eval_with(delegate, &script, &args)?;
            Ok(())
        }

        #[ink(message)]
        pub fn run_js_using_delegate(&self, script: String, args: Vec<String>) -> Result<(), String> {
            let delegate = get_driver("JsDelegate")?;
            phat_js::eval_with(delegate, &script, &args)?;
            Ok(())
        }
    }

    pub fn get_driver(name: &str) -> Result<Hash, String> {
        use phat_js::ConvertTo;
        let system = pink::system::SystemRef::instance();
        let delegate = system.get_driver(name.into()).ok_or("No JS driver found")?;
        Ok(delegate.convert_to())
    }
}
