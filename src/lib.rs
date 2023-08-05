extern crate alloc;

pub use service::Service;
pub use service_keeper::ServiceKeeper;

mod host_functions;
mod service;
mod service_keeper;

mod traits;

#[cfg(feature = "native")]
pub mod runtime {
    pub use tokio::main;
    pub(crate) use {
        hyper::client::HttpConnector,
        tokio::{task::spawn_local as spawn, time},
    };
    pub(crate) fn getrandom(buf: &mut [u8]) -> Option<()> {
        use rand::RngCore;
        rand::thread_rng().fill_bytes(buf);
        Some(())
    }
    pub(crate) type AccountId = [u8; 32];
    pub(crate) struct HyperExecutor;
    impl<F: core::future::Future + 'static> hyper::rt::Executor<F> for HyperExecutor {
        fn execute(&self, fut: F) {
            spawn(fut);
        }
    }

    pub async fn main_loop() {
        env_logger::init();
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            })
            .await
    }
}

#[cfg(not(feature = "native"))]
pub mod runtime {
    pub(crate) use sidevm::{
        env::messages::AccountId, exec::HyperExecutor, net::HttpConnector, ocall::getrandom, spawn,
        time,
    };

    pub use sidevm::main;

    pub async fn main_loop() {
        use log::info;

        sidevm::logger::Logger::with_max_level(log::LevelFilter::Debug).init();

        info!("Starting sidevm...");
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
}
