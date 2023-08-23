extern crate alloc;

pub use service::Service;
pub use service_keeper::ServiceKeeper;

mod host_functions;
mod service;
mod service_keeper;

mod traits;

#[cfg(feature = "native")]
pub mod runtime {
    use hyper::client::HttpConnector;
    use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};

    use log::info;
    pub use tokio::main;
    pub use tokio::{task::spawn_local as spawn, time};
    pub fn http_connector() -> HttpsConnector<HttpConnector> {
        HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .build()
    }
    pub fn getrandom(buf: &mut [u8]) -> Option<()> {
        use rand::RngCore;
        rand::thread_rng().fill_bytes(buf);
        Some(())
    }
    pub type AccountId = [u8; 32];
    pub struct HyperExecutor;
    impl<F: core::future::Future + 'static> hyper::rt::Executor<F> for HyperExecutor {
        fn execute(&self, fut: F) {
            spawn(fut);
        }
    }
    pub async fn run_local<F: core::future::Future>(fut: F) -> F::Output {
        let local = tokio::task::LocalSet::new();
        local.run_until(fut).await
    }
    pub async fn main_loop() {
        let script_file = std::env::args()
            .nth(1)
            .expect("Please provide a script file as the first argument");
        let script = std::fs::read_to_string(script_file).expect("Failed to read script file");
        let service = crate::Service::new_ref();
        let output = service.exec_script(&script);
        match output {
            Ok(value) if value.is_undefined() => {}
            _ => {
                info!("Script output: {:?}", output);
            }
        }
        if service.number_of_tasks() > 0 {
            service.wait_for_tasks().await;
        }
    }
    pub use tracing_subscriber::fmt::init as init_logger;
}

#[cfg(not(feature = "native"))]
pub mod runtime {
    pub use sidevm::{
        env::messages::AccountId, exec::HyperExecutor, net::HttpConnector, ocall::getrandom, spawn,
        time,
    };

    pub use sidevm::main;

    pub fn http_connector() -> HttpConnector {
        HttpConnector::new()
    }
    pub async fn main_loop() {
        use log::info;
        loop {
            tokio::select! {
                query = sidevm::channel::incoming_queries().next() => {
                    let Some(query) = query else {
                        info!("Host dropped the channel, exiting...");
                        break;
                    };
                    let reply = crate::ServiceKeeper::handle_query(query.origin, &query.payload);
                    _ = query.reply_tx.send(&reply);
                }
                message = sidevm::channel::input_messages().next() => {
                    let Some(message) = message else {
                        info!("Host dropped the channel, exiting...");
                        break;
                    };
                    crate::ServiceKeeper::handle_message(message);
                }
            }
        }
    }
    pub async fn run_local<F: core::future::Future>(fut: F) -> F::Output {
        fut.await
    }
    pub fn init_logger() {
        sidevm::logger::Logger::with_max_level(log::LevelFilter::Debug).init();
    }
}
