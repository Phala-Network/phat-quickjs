#![cfg_attr(not(feature = "std"), no_std, no_main)]
//! Each pink contract can have a backgroud sidevm instance associated with it.
//! Here demonstrates how to interact with the background sidevm quickjs in a pink contract.

#[macro_use]
extern crate alloc;

pub use control::*;

#[ink::contract]
mod control {
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    enum SidevmMessage {
        Run {
            name: String,
            source: String,
            reset: bool,
        },
    }

    #[ink(storage)]
    pub struct Control {
        engine_code_hash: [u8; 32],
        config: u32,
        script: String,
    }

    impl Control {
        #[ink(constructor)]
        pub fn new(engine_code_hash: [u8; 32]) -> Self {
            Self {
                config: 0,
                engine_code_hash,
                script: "console.log('Hello, SideVM!')".to_string(),
            }
        }

        /// Start the sidevm instance associated with this contract.
        #[ink(message)]
        pub fn restart_sidevm(&self) {
            pink::start_sidevm(self.engine_code_hash).expect("failed to start sidevm");
        }

        /// Stop the sidevm instance associated with this contract.
        #[ink(message)]
        pub fn stop_sidevm(&self) {
            pink::force_stop_sidevm();
        }

        /// The the sidevm instance startup, it will query this method for the init scripts
        #[ink(message)]
        pub fn sidevm_init_scripts(&self) -> Vec<String> {
            // The init script to setup the globalThis.config in the JS instance.
            let init = format!("globalThis.config = {};", self.config);
            vec![
                init,
                self.script.clone(), // The script to run in the JS instance.
            ]
        }

        /// Update the secret config of the JavaScript.
        #[ink(message)]
        pub fn update_config(&mut self, config: u32) {
            self.config = config;
            // Update the config in the sidevm instance via run a JS script on an existing JS VM instance.
            let message = SidevmMessage::Run {
                // '_' is the default JS instance name.
                name: "_".to_string(),
                // update the config in the JS instance.
                source: format!(
                    r#"
                    globalThis.config = {config};
                    console.log('Config updated to {config}');
                    "#
                ),
                // Don't reset the JS instance before run.
                reset: false,
            };
            let encoded = pink_json::to_vec(&message).expect("encode sidevm message failed");
            pink::push_sidevm_message(encoded);
        }

        /// Get the config for testing.
        #[ink(message)]
        pub fn get_config(&self) -> u32 {
            self.config
        }

        /// Update the source code of the JavaScript.
        #[ink(message)]
        pub fn set_script(&mut self, script: String) {
            self.script = script;
        }

        /// Update the engine code hash of the SideVM.
        #[ink(message)]
        pub fn update_engine(&mut self, engine_code_hash: [u8; 32]) {
            self.engine_code_hash = engine_code_hash;
        }
    }
}
