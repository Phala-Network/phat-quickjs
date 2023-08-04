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
        console.log('Hello, world!')
        function test(n) {
            console.log("test", n, Math.random());
        }
        const id = setInterval(test, 1000, 42);
        setTimeout(() => clearInterval(0), 3000);
        setInterval("hello", 4000);
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
