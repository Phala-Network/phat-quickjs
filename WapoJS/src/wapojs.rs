#![cfg_attr(not(feature = "native"), no_main)]

extern crate alloc;

use wapo_quickjs::{js_eval, runtime};

#[runtime::main]
async fn main() {
    runtime::init_logger();
    let _ = js_eval::run(std::env::args().skip(1))
        .await
        .expect("failed to run js code");
}
