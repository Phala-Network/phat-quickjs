use std::net::SocketAddr;

use anyhow::Result;
pub use wapo::channel::HttpRequest;
pub use wapo::env::messages::{HttpHead, HttpResponseHead};
pub use wapo::net::TcpStream;
pub use wapo::ocall;
pub use wapo::{
    env::messages::AccountId, hyper_rt::HyperExecutor, net::hyper_v0::HttpConnector,
    ocall::getrandom, spawn, time,
};

pub use wapo::main;
pub use wapo::net::SniTlsListener as TlsListener;
pub use wapo::net::TcpListener;

pub fn http_connector() -> HttpConnector {
    HttpConnector::new()
}

pub fn init_logger() {
    wapo::logger::init();
}
pub fn set_output(output: Vec<u8>) {
    wapo::ocall::emit_program_output(&output).expect("failed to emit program output")
}
pub async fn run_local<F: core::future::Future>(fut: F) -> F::Output {
    fut.await
}

pub fn sni_listen(sni: &str, cert: &str, key: &str) -> Result<TlsListener> {
    let tls_config = wapo::env::tls::TlsServerConfig::V0 {
        cert: cert.to_string(),
        key: key.to_string(),
    };
    Ok(TlsListener::bind(sni, tls_config)?)
}

pub async fn tcp_accept(listener: &TcpListener) -> Result<(TcpStream, SocketAddr)> {
    listener
        .accept()
        .await
        .map_err(|e| anyhow::anyhow!("failed to accept tcp connection: {e}"))
}
