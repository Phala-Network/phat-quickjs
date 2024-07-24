use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use tokio::time::sleep;
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};
use wapod::{
    config::{Paths, WorkerConfig},
    types::ticket::AppManifest,
    WorkerArgs,
};

use config::Config;
mod config;
mod web_api;

type Worker = wapod::Worker<Config>;

#[derive(Parser, Clone, Debug)]
#[clap(about = "wapojs", version, author)]
pub struct Args {
    /// Maximum memory size for each instance.
    #[arg(long, short = 'm', default_value = "128M", value_parser = parse_size)]
    memory_size: u64,
    /// Maximum size for each payload.
    #[arg(long, default_value = "1M", value_parser = parse_size)]
    query_size: u64,
    /// Port number for the user service to listen on.
    #[arg(long, short = 'p', default_value = "8002")]
    port: u16,
    /// Port number for the user service to listen on.
    #[arg(long, default_value = "8443")]
    tls_port: u16,
    /// Verify the server certificate chain when the guest tries to listen on an SNI.
    #[arg(long)]
    verify_cert: bool,
    /// The wasmtime compiler to use
    #[arg(long, short = 'c')]
    wasm_compiler: Option<String>,
    /// The WASM file of the JS engine
    #[arg(long, short = 'e')]
    engine: Option<String>,
    /// Remember the engine code for future use
    #[arg(long, short = 'u')]
    save_engine: bool,
    /// The JS script to run
    script: String,
    /// The rest of the arguments are passed to the WASM program
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    args: Vec<String>,
}

fn parse_size(input: &str) -> Result<u64, parse_size::Error> {
    parse_size::Config::new().with_binary().parse_size(input)
}

fn init_logger() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info,rocket=warn");
    }
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    info!("starting wapojs, data dir: {:?}", Config::data_dir());
    let args = Args::parse();
    <Config as WorkerConfig>::Paths::create_dirs_if_needed()
        .context("failed to create directories")?;
    let use_winch = match args.wasm_compiler.as_deref() {
        Some("winch") => true,
        Some("cranelift") => false,
        Some("auto") => false,
        None => true,
        _ => return Err(anyhow!("invalid wasm compiler")),
    };
    let worker_args = WorkerArgs::builder()
        .instance_memory_size(args.memory_size)
        .max_instances(1)
        .module_cache_size(1)
        .no_mem_pool(true)
        .use_winch(use_winch)
        .tcp_listen_port_range(0..=65535)
        .tls_port(Some(args.tls_port))
        .verify_tls_server_cert(args.verify_cert)
        .on_demand_connection_timeout(Duration::from_secs(10))
        .build();
    let Some(engine_file) = args.engine.clone().or_else(config::read_default_engine) else {
        return Err(anyhow!(
            "no js engine provided, use --engine or set the default engine"
        ));
    };
    info!("using engine file: {engine_file}");
    let engine_code = std::fs::read(&engine_file).context("failed to read engine code")?;
    if args.save_engine {
        let engine_file = std::fs::canonicalize(&engine_file).expect("canonicalize");
        config::save_default_engine(&engine_file)?;
    }
    let script = std::fs::read_to_string(&args.script).context("failed to read the script")?;
    let worker = Worker::create_running(worker_args)
        .await
        .context("failed to create worker state")?;

    let mut instance_args = vec!["-c".to_string(), script];
    instance_args.extend(args.args.into_iter());

    let code_hash = worker
        .blob_loader()
        .put("sha256:", &mut &engine_code[..])
        .await
        .context("failed to upload engine code")?;
    let manifest = AppManifest {
        version: 1,
        code_hash,
        args: instance_args,
        env_vars: std::env::vars().collect(),
        on_demand: false,
        resizable: false,
        max_query_size: args.query_size.try_into().context("invalid query size")?,
        label: "test".to_string(),
        required_blobs: vec![],
    };
    let app_info = worker
        .deploy_app(manifest, false, true)
        .await
        .context("failed to deploy the app")?;
    info!(
        "app deployed at address: 0x{:?}",
        hex_fmt::HexFmt(app_info.address)
    );
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let server = web_api::serve_user(worker.clone(), args.port);
    tokio::spawn(async move {
        loop {
            if worker.info(false).running_instances == 0 {
                stop_tx.send(()).ok();
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }
    });
    tokio::select! {
        _ = stop_rx => {}
        _ = server => {}
    }
    Ok(())
}
