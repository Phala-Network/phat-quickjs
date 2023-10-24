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
    use js::ToJsValue;
    pub use sidevm::env::messages::{HttpHead, HttpResponseHead};
    use tokio::io::DuplexStream;

    pub struct HttpRequest {
        /// The HTTP request head.
        pub head: HttpHead,
        /// The IO stream to read the request body and write the response body.
        pub io_stream: DuplexStream,
        /// The reply channel to send the response head.
        pub response_tx: tokio::sync::oneshot::Sender<HttpResponseHead>,
    }

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
        let args: Vec<_> = std::env::args().collect();
        let script_file = args
            .get(1)
            .expect("Please provide a script file as the first argument");
        let source = if script_file.starts_with("#") {
            script_file.trim_start_matches(|c| c != '\r' && c != '\n')
        } else {
            script_file
        };
        let script = std::fs::read_to_string(source).expect("Failed to read script file");
        let service = crate::Service::new_ref();
        let script_args = &args[2..];
        let js_ctx = service.context();
        let js_args = script_args
            .to_js_value(&js_ctx)
            .expect("Failed to convert args to js value");
        js_ctx
            .get_global_object()
            .set_property("scriptArgs", &js_args)
            .expect("Failed to set scriptArgs");
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
    use log::{error, info};
    use scale::Decode;
    pub use sidevm::channel::HttpRequest;
    pub use sidevm::env::messages::{HttpHead, HttpResponseHead};
    use sidevm::local_contract::query_pink;
    pub use sidevm::{
        env::messages::AccountId, exec::HyperExecutor, net::HttpConnector, ocall::getrandom, spawn,
        time,
    };

    pub use sidevm::main;

    pub fn http_connector() -> HttpConnector {
        HttpConnector::new()
    }

    async fn get_init_script() -> Option<String> {
        type LangError = u8;
        let myid = sidevm::ocall::vmid().ok()?;
        let selector = ink_macro::selector_bytes!("sidevm_init_script");
        let result = query_pink(myid, selector.to_vec()).await;
        let result = match result {
            Ok(response) => Result::<String, LangError>::decode(&mut &response[..]).ok()?,
            Err(err) => {
                error!("Failed to query init script: {err:?}");
                return None;
            }
        };
        match result {
            Ok(script) => Some(script),
            Err(err) => {
                error!("Failed to get init script, error code: {err}");
                None
            }
        }
    }

    pub async fn main_loop() {
        info!("Getting init script...");
        match get_init_script().await {
            None => {
                info!("No init script found, starting the service keeper...");
            }
            Some(script) => {
                info!("Executing init script...");
                crate::ServiceKeeper::exec_script("_main", &script);
            }
        }
        info!("Listening for incoming queries...");
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
                connection = sidevm::channel::incoming_http_connections().next() => {
                    let Some(connection) = connection else {
                        info!("Host dropped the channel, exiting...");
                        break;
                    };
                    if let Err(err) = crate::ServiceKeeper::handle_connection(connection) {
                        error!("Failed to handle incoming http connection: {err:?}");
                    }
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
