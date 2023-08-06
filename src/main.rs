#![cfg_attr(not(feature = "native"), no_main)]

extern crate alloc;

use log::info;
use quickjs::{runtime, Service};

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
            setTimeout(() => {
                console.log("Hello, world! 2");
            }, 2000);
            const chunks = [];
            async function test() {
                console.log("entered test");
                const response = await fetch("https://www.baidu.com");
                console.log("status:", response.status);
                console.log("statusText:", response.statusText);
                const bodyReader = response.body.getReader();
                while(true) {
                    const {done, value} = await bodyReader.read();
                    if (done) {
                        break;
                    }
                    console.log("chunk:", value);
                }
            }
            test()
        "#;
        info!("Compiling script...");
        let code = qjs_sys::compile(test_script, "test.js").unwrap();
        info!("Executing script...");
        let rv = service.exec_bytecode(&code);
        info!("Script executed: {:?}", rv);

        runtime::main_loop().await;
        info!("Exiting...");
    })
    .await;
}

#[cfg(not(feature = "native"))]
#[no_mangle]
extern "C" fn __main_argc_argv(_argc: i32, _argv: *const *const u8) -> i32 {
    0
}
