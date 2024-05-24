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
}
