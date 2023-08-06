#![cfg_attr(not(feature = "native"), no_main)]

extern crate alloc;

use quickjs::{runtime, Service};

#[runtime::main]
async fn main() {
    runtime::init_logger();
    runtime::run_local(async {
        // for test
        let service = Service::new_ref();
        let test0 = r#"
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
        let _ = service.exec_script(test0);
        runtime::main_loop().await
    })
    .await;
}

#[cfg(not(feature = "native"))]
#[no_mangle]
extern "C" fn __main_argc_argv(_argc: i32, _argv: *const *const u8) -> i32 {
    0
}
