extern crate alloc;

use core::ffi::CStr;
use log::info;
use sidevm::logger::Logger;

use service_keeper::ServiceKeeper;

mod host_functions;
mod service;
mod service_keeper;

#[sidevm::main]
async fn main() {
    Logger::with_max_level(log::LevelFilter::Debug).init();

    info!("Starting sidevm...");

    let service = service::Service::new_ref("test");
    let ret = service.exec_script(
        r#"
        console.log('Hello, world!')
        function test(n) {
            console.log("test", n, Math.random());
        }
        setInterval(test, 1000, 42);
        "#,
    );
    loop {
        tokio::select! {
            query = sidevm::channel::incoming_queries().next() => {
                let Some(query) = query else {
                    info!("Host dropped the channel, exiting...");
                    break;
                };
                let reply = ServiceKeeper::handle_query(query.origin, &query.payload);
                _ = query.reply_tx.send(&reply);
            }
            message = sidevm::channel::input_messages().next() => {
                let Some(message) = message else {
                    info!("Host dropped the channel, exiting...");
                    break;
                };
                ServiceKeeper::handle_message(message);
            }
        }
    }
}
