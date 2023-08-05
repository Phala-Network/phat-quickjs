extern crate alloc;

use log::info;
use sidevm::logger::Logger;

use service_keeper::ServiceKeeper;

mod host_functions;
mod service;
mod service_keeper;

mod traits;

#[sidevm::main]
async fn main() {
    Logger::with_max_level(log::LevelFilter::Debug).init();

    info!("Starting sidevm...");

    // for test
    let service = service::Service::new_ref();
    let _ = service.exec_script(
        r#"
        const chunks = [];
        async function test() {
            console.log("test");
            const response = await fetch("https://www.baidu.com/abc");
            console.log("status:", response.status);
            console.log("statusText:", response.statusText);
            const body = await response.text();
            // print in chunks of 128 bytes
            for (let i = 0; i < body.length; i += 128) {
                console.log(body.slice(i, i + 128));
            }
        }
        test()
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
