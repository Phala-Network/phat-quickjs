extern crate alloc;

pub use service::Service;
pub use service_keeper::ServiceKeeper;

mod host_functions;
mod service;
mod service_keeper;

pub mod js_eval;
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
    pub fn set_output(_output: Vec<u8>) {}
}

#[cfg(feature = "sidevm")]
pub mod runtime {
    use anyhow::{anyhow, Context, Result};
    use log::{error, info, warn};
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

    async fn get_init_scripts() -> Result<Vec<String>> {
        type LangError = u8;
        let myid = sidevm::ocall::vmid()?;
        let selector = ink_macro::selector_bytes!("sidevm_init_scripts");
        let response = query_pink(myid, selector.to_vec())
            .await
            .map_err(|err| anyhow!("Failed to query init script: {err:?}"))?;
        let scripts = Result::<Vec<String>, LangError>::decode(&mut &response[..])
            .context("Failed to decode Result::<String, LangError>")?
            .map_err(|err| anyhow!("LangError({err})"))?;
        Ok(scripts)
    }

    pub async fn main_loop() {
        info!("Getting init scripts...");
        match get_init_scripts().await {
            Err(err) => {
                warn!("Failed to get init scripts: {err}");
                info!("No init script found, starting the service keeper...");
            }
            Ok(scripts) => {
                let n = scripts.len();
                for (i, script) in scripts.iter().enumerate() {
                    info!("Executing init script {}/{n}...", i + 1);
                    crate::ServiceKeeper::exec_script("_", &script);
                }
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
        use sidevm::logger::Logger;
        static LOGGER: Logger = Logger::with_max_level(log::LevelFilter::Info);
        LOGGER.init();
    }
    pub fn set_output(output: Vec<u8>) {
        sidevm::ocall::emit_program_output(&output).expect("Failed to emit program output")
    }
}

#[cfg(feature = "web")]
pub mod runtime {
    pub struct HttpRequest;
    pub use sidevm::env::messages::{AccountId, HttpHead, HttpResponseHead};
    pub use wasm_bindgen_futures::spawn_local as spawn;

    use log::{LevelFilter, Log, Metadata};
    use wasm_bindgen::JsValue as WebJsValue;
    use web_sys::console;

    struct Logger {
        max_level: LevelFilter,
    }

    impl Logger {
        /// Create a new logger with the given maximum level.
        pub const fn with_max_level(max_level: LevelFilter) -> Self {
            Self { max_level }
        }

        /// Install the logger as the global logger.
        pub fn init(&'static self) {
            log::set_max_level(self.max_level);
            log::set_logger(self).unwrap();
        }
    }

    impl Log for Logger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.level() <= self.max_level
        }

        fn log(&self, record: &log::Record) {
            if self.enabled(record.metadata()) {
                let message = WebJsValue::from_str(&format!("{}", record.args()));
                match record.level() {
                    log::Level::Error => console::error_1(&message),
                    log::Level::Warn => console::warn_1(&message),
                    log::Level::Info => console::info_1(&message),
                    log::Level::Debug => console::debug_1(&message),
                    log::Level::Trace => console::trace_1(&message),
                }
            }
        }

        fn flush(&self) {}
    }

    pub fn init_logger() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        static LOGGER: Logger = Logger::with_max_level(log::LevelFilter::Debug);
        ONCE.call_once(|| {
            LOGGER.init();
        });
    }

    pub fn getrandom(buf: &mut [u8]) -> Result<(), WebJsValue> {
        buf.iter_mut().for_each(|byte| {
            *byte = (js_sys::Math::random() * 256.0) as u8;
        });
        Ok(())
    }

    pub mod time {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen_futures::JsFuture;

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(catch, js_name=setTimeout)]
            fn set_timeout(handler: &::js_sys::Function, timeout: i32) -> Result<i32, JsValue>;
        }

        fn js_sleep(ms: i32) -> JsFuture {
            JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
                set_timeout(&resolve, ms).expect("Failed to set timeout");
            }))
        }

        pub async fn sleep(duration: std::time::Duration) {
            js_sleep(duration.as_millis() as i32).await.unwrap();
        }
    }

    #[no_mangle]
    extern "C" fn __pink_clock_time_get(_id: u32, _precision: u64, _retptr0: *mut u64) -> u16 {
        0
    }

    #[no_mangle]
    extern "C" fn __pink_fd_write(
        fd: core::ffi::c_int,
        buf: *const core::ffi::c_uchar,
        len: usize,
    ) -> usize {
        // Bridge the fd to the console.
        let buf = unsafe { std::slice::from_raw_parts(buf, len) };
        let buf = String::from_utf8_lossy(buf).into_owned();
        match fd {
            1 => console::log_2(&"JS:".to_string().into(), &buf.into()),
            2 => console::error_2(&"JS:".to_string().into(), &buf.into()),
            _ => unimplemented!("Unsupported fd: {fd}"),
        }
        len
    }
}
