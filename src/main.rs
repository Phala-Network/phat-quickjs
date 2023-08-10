#![cfg_attr(not(feature = "native"), no_main)]

extern crate alloc;

use sidevm_quickjs::runtime;

#[runtime::main]
async fn main() {
    runtime::init_logger();
    runtime::run_local(async {
        runtime::main_loop().await;
    })
    .await;
}

#[cfg(not(feature = "native"))]
#[no_mangle]
extern "C" fn __main_argc_argv(_argc: i32, _argv: *const *const u8) -> i32 {
    0
}
