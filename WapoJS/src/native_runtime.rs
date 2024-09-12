use std::net::SocketAddr;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use hyper::client::HttpConnector;
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use sni_tls_listener::{Agent, Generate as _, SniTlsListener, Subscription};
use tokio::io::DuplexStream;
pub use tokio::net::TcpListener;
use tokio_rustls::rustls::ClientConfig;
pub use wapo::env::messages::{HttpHead, HttpResponseHead};

static WAPO_SNI_TLS_PORT: AtomicU16 = AtomicU16::new(443);

pub fn set_sni_tls_port(port: u16) {
    WAPO_SNI_TLS_PORT.store(port, Ordering::Relaxed);
}

pub struct TlsListener(Subscription);

fn global_sni_listener() -> Result<&'static Agent> {
    static GLOBAL_SNI_LISTENER: OnceLock<Result<(SniTlsListener, Agent)>> = OnceLock::new();
    let sni_tls_port = WAPO_SNI_TLS_PORT.load(Ordering::Relaxed);
    let listener = GLOBAL_SNI_LISTENER.get_or_init(|| {
        SniTlsListener::install_ring_provider();
        let listener =
            futures::executor::block_on(SniTlsListener::bind("0.0.0.0", sni_tls_port, false))?;
        let agent = listener.agent(|| {}, true, Duration::from_micros(0));
        Ok((listener, agent))
    });
    match listener {
        Ok((_listener, agent)) => Ok(agent),
        Err(err) => Err(anyhow!("failed to bind SNI listener: {err}")),
    }
}

pub fn sni_listen(sni: &str, cert: &str, key: &str) -> Result<TlsListener> {
    let listener = global_sni_listener()?;
    let key = sni_tls_listener::wrap_certified_key(cert.as_bytes(), key.as_bytes())
        .context("invalid cert or key")?;
    log::info!(target: "wapo::tls", "SNI listener is listening on {sni}");
    let subscription = listener.subscribe(sni, key)?;
    Ok(TlsListener(subscription))
}

impl TlsListener {
    pub async fn accept(&mut self) -> Result<(TcpStream, SocketAddr)> {
        let conn = self.0.next().await.ok_or(anyhow!("listener closed"))?;
        Ok((TcpStream::ServerTlsSteam(conn.stream), conn.remote_addr))
    }
}

pub async fn tcp_accept(listener: &TcpListener) -> Result<(TcpStream, SocketAddr)> {
    let (tcp, addr) = listener
        .accept()
        .await
        .map_err(|e| anyhow::anyhow!("failed to accept tcp connection: {e}"))?;
    Ok((TcpStream::TcpStream(tcp), addr))
}

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
    ClientTlsSteam(tokio_rustls::client::TlsStream<tokio::net::TcpStream>),
    ServerTlsSteam(tokio_rustls::server::TlsStream<tokio::net::TcpStream>),
}

impl TcpStream {
    pub async fn connect(host: &str, port: u16, enable_tls: bool) -> Result<TcpStream> {
        let stream = tokio::net::TcpStream::connect((host, port)).await?;
        if enable_tls {
            let connector = tokio_rustls::TlsConnector::from(default_client_config());
            let server_name = host.to_string().try_into().context("invalid server name")?;
            let stream = connector.connect(server_name, stream).await?;
            Ok(TcpStream::ClientTlsSteam(stream))
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
            TcpStream::ClientTlsSteam(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
            TcpStream::ServerTlsSteam(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
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
            TcpStream::ClientTlsSteam(stream) => {
                std::pin::Pin::new(stream).poll_write_vectored(cx, bufs)
            }
            TcpStream::ServerTlsSteam(stream) => {
                std::pin::Pin::new(stream).poll_write_vectored(cx, bufs)
            }
        }
    }

    fn is_write_vectored(&self) -> bool {
        match self {
            TcpStream::TcpStream(stream) => stream.is_write_vectored(),
            TcpStream::ClientTlsSteam(stream) => stream.is_write_vectored(),
            TcpStream::ServerTlsSteam(stream) => stream.is_write_vectored(),
        }
    }

    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            TcpStream::TcpStream(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
            TcpStream::ClientTlsSteam(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
            TcpStream::ServerTlsSteam(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            TcpStream::TcpStream(stream) => std::pin::Pin::new(stream).poll_flush(cx),
            TcpStream::ClientTlsSteam(stream) => std::pin::Pin::new(stream).poll_flush(cx),
            TcpStream::ServerTlsSteam(stream) => std::pin::Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            TcpStream::TcpStream(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
            TcpStream::ClientTlsSteam(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
            TcpStream::ServerTlsSteam(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
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

pub mod ocall {
    use anyhow::Result;
    use std::collections::BTreeSet;
    use std::sync::Mutex;

    static LOCKS: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

    pub fn sign(_message: &[u8]) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    pub fn worker_pubkey() -> Result<[u8; 32]> {
        Ok([0; 32])
    }

    pub fn sgx_quote(_message: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    pub fn read_boot_data() -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    pub fn write_boot_data(_data: &[u8]) -> Result<()> {
        Ok(())
    }

    pub fn app_try_lock(path: &str) -> Result<()> {
        let locks = &mut LOCKS.lock().unwrap();
        if locks.contains(path) {
            anyhow::bail!("lock already held");
        }
        locks.insert(path.to_string());
        Ok(())
    }

    pub fn app_unlock(path: &str) -> Result<()> {
        let locks = &mut LOCKS.lock().unwrap();
        if locks.remove(path) {
            Ok(())
        } else {
            anyhow::bail!("lock not held");
        }
    }
}
