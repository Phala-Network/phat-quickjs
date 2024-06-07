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
    pub use wapo::net::TcpStream;
    pub use wapo::{
        env::messages::AccountId, hyper_rt::HyperExecutor, net::hyper_v0::HttpConnector,
        ocall::getrandom, spawn, time,
    };

    pub use wapo::main;

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
}

#[cfg(feature = "native")]
pub mod runtime {
    use std::sync::{Arc, OnceLock};

    use anyhow::{Context, Result};
    use hyper::client::HttpConnector;
    use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
    use tokio::io::DuplexStream;
    use tokio_rustls::rustls::ClientConfig;
    pub use wapo::env::messages::{HttpHead, HttpResponseHead};

    fn default_client_config() -> Arc<ClientConfig> {
        static CLIENT_CONFIG: OnceLock<Arc<ClientConfig>> = OnceLock::new();
        CLIENT_CONFIG
            .get_or_init(|| {
                let root_store = tokio_rustls::rustls::RootCertStore::from_iter(
                    webpki_roots::TLS_SERVER_ROOTS.iter().cloned(),
                );
                let config = ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth();
                Arc::new(config)
            })
            .clone()
    }

    pub enum TcpStream {
        TcpStream(tokio::net::TcpStream),
        TlsSteam(tokio_rustls::client::TlsStream<tokio::net::TcpStream>),
    }

    impl TcpStream {
        pub async fn connect(host: &str, port: u16, enable_tls: bool) -> Result<TcpStream> {
            let stream = tokio::net::TcpStream::connect((host, port)).await?;
            if enable_tls {
                let connector = tokio_rustls::TlsConnector::from(default_client_config());
                let server_name = host.to_string().try_into().context("invalid server name")?;
                let stream = connector.connect(server_name, stream).await?;
                Ok(TcpStream::TlsSteam(stream))
            } else {
                Ok(TcpStream::TcpStream(stream))
            }
        }
    }

    impl tokio::io::AsyncRead for TcpStream {
        fn poll_read(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match self.get_mut() {
                TcpStream::TcpStream(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
                TcpStream::TlsSteam(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
            }
        }
    }

    impl tokio::io::AsyncWrite for TcpStream {
        fn poll_write_vectored(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            bufs: &[std::io::IoSlice<'_>],
        ) -> std::task::Poll<Result<usize, std::io::Error>> {
            match self.get_mut() {
                TcpStream::TcpStream(stream) => {
                    std::pin::Pin::new(stream).poll_write_vectored(cx, bufs)
                }
                TcpStream::TlsSteam(stream) => {
                    std::pin::Pin::new(stream).poll_write_vectored(cx, bufs)
                }
            }
        }

        fn is_write_vectored(&self) -> bool {
            match self {
                TcpStream::TcpStream(stream) => stream.is_write_vectored(),
                TcpStream::TlsSteam(stream) => stream.is_write_vectored(),
            }
        }

        fn poll_write(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<Result<usize, std::io::Error>> {
            match self.get_mut() {
                TcpStream::TcpStream(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
                TcpStream::TlsSteam(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
            }
        }

        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), std::io::Error>> {
            match self.get_mut() {
                TcpStream::TcpStream(stream) => std::pin::Pin::new(stream).poll_flush(cx),
                TcpStream::TlsSteam(stream) => std::pin::Pin::new(stream).poll_flush(cx),
            }
        }

        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), std::io::Error>> {
            match self.get_mut() {
                TcpStream::TcpStream(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
                TcpStream::TlsSteam(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
            }
        }
    }

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

fn todo() {
    let todo = "js::Bytes support for ArrayBuffer";
}