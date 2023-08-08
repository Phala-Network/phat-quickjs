#![cfg_attr(not(feature = "native"), no_main)]

extern crate alloc;

use log::info;
use sidevm_quickjs::{runtime, Service};

#[runtime::main]
async fn main() {
    runtime::init_logger();
    info!("Starting...");
    runtime::run_local(async {
        // for test
        let service = Service::new_ref();
        runtime::time::sleep(std::time::Duration::from_secs(1)).await;
        let test_script = r#"
            console.log("Hello, world!");
            const chunks = [];
            async function test() {
                console.log("entered test");
                const response = await fetch("https://www.baidu.com");
                console.log("status:", response.status);
                console.log("statusText:", response.statusText);
                const blob = await response.blob();
                console.log("blob:", blob.size);
            }
            test()
        "#;
        info!("Executing script...");
        let rv = service.exec_script(test_script);
        info!("Script executed: {:?}", rv);

        // runtime::main_loop().await;
        runtime::time::sleep(std::time::Duration::from_secs(2)).await;
        info!("Exiting...");
    })
    .await;
}

#[cfg(not(feature = "native"))]
#[no_mangle]
extern "C" fn __main_argc_argv(_argc: i32, _argv: *const *const u8) -> i32 {
    0
}
