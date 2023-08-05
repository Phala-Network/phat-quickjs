#![cfg_attr(not(feature = "native"), no_main)]

extern crate alloc;

use quickjs::{runtime, Service};

#[runtime::main]
async fn main() {
    // for test
    let service = Service::new_ref();
    let _ = service.exec_script(
        r#"
        console.log("Hello, world!");
        console.log("ReadableStream:", ReadableStream);
        const chunks = [];
        async function test() {
            console.log("test");
            const response = await fetch("https://www.baidu.com");
            console.log("status:", response.status);
            console.log("statusText:", response.statusText);
            const body = await response.body();
            for await (const chunk of body) {
                console.log("chunk:", chunk);
            }
        }
        test()
        "#,
    );

    runtime::main_loop().await
}

#[cfg(not(feature = "native"))]
#[no_mangle]
extern "C" fn __main_argc_argv(_argc: i32, _argv: *const *const u8) -> i32 {
    0
}
