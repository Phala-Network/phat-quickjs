extern crate alloc;

pub use service::Service;

mod host_functions;
mod service;

pub mod js_eval;
mod traits;

#[cfg(feature = "wapo")]
pub mod runtime {
    pub use wapo::channel::HttpRequest;
    pub use wapo::env::messages::{HttpHead, HttpResponseHead};
    pub use wapo::{
        env::messages::AccountId, hyper_rt::HyperExecutor, net::hyper_v0::HttpConnector,
        ocall::getrandom, spawn, time,
    };

    pub use wapo::main;

    pub fn http_connector() -> HttpConnector {
        HttpConnector::new()
    }

    pub fn init_logger() {
        use wapo::logger::Logger;
        Logger::with_max_level(log::LevelFilter::Info).init();
    }
    pub fn set_output(output: Vec<u8>) {
        wapo::ocall::emit_program_output(&output).expect("failed to emit program output")
    }
    pub async fn run_local<F: core::future::Future>(fut: F) -> F::Output {
        fut.await
    }
}

#[cfg(feature = "native")]
pub mod runtime {
    use hyper::client::HttpConnector;
    use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
    pub use wapo::env::messages::{HttpHead, HttpResponseHead};
    use tokio::io::DuplexStream;

    pub struct HttpRequest {
        /// The HTTP request head.
        pub head: HttpHead,
        /// The IO stream to read the request body and write the response body.
        pub io_stream: DuplexStream,
        /// The reply channel to send the response head.
        pub response_tx: tokio::sync::oneshot::Sender<HttpResponseHead>,
    }

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
    pub use tracing_subscriber::fmt::init as init_logger;
    pub fn set_output(_output: Vec<u8>) {}
}