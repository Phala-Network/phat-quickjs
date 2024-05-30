use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use tokio::time::sleep;
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};
use wapod::{
    config::{Paths, WorkerConfig},
    rpc::prpc::{Manifest, StringPair},
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

fn engine_config_file() -> Option<std::path::PathBuf> {
    Some(dirs::config_dir()?.join("wapojs").join("default_engine"))
}

fn read_default_engine() -> Option<String> {
    engine_config_file().and_then(|s| std::fs::read_to_string(s).ok())
}

fn save_default_engine(engine: &str) -> Result<()> {
    let path = engine_config_file().context("failed to get config directory")?;
    std::fs::create_dir_all(path.parent().context("no parent")?).context("failed to create dir")?;
    std::fs::write(path, engine).context("failed to write engine code")
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    let args = Args::parse();
    <Config as WorkerConfig>::Paths::create_dirs_if_needed()
        .context("failed to create directories")?;

    let worker_args = WorkerArgs::builder()
        .instance_memory_size(args.memory_size)
        .max_instances(1)
        .module_cache_size(1)
        .no_mem_pool(true)
        .use_winch(true)
        .build();
    let Some(engine_file) = args.engine.clone().or_else(read_default_engine) else {
        return Err(anyhow!(
            "no js engine provided, use --engine or set the default engine"
        ));
    };
    let engine_code = std::fs::read(&engine_file).context("failed to read engine code")?;
    if args.save_engine {
        save_default_engine(&engine_file)?;
    }
    let script = std::fs::read_to_string(&args.script).context("failed to read engine code")?;
    let worker = Worker::crate_running(worker_args).context("failed to create worker state")?;
    let hash_algorithm = "sha256".to_string();

    let mut instance_args = vec!["-c".to_string(), script];
    instance_args.extend(args.args.into_iter());

    let code_hash = worker
        .blob_loader()
        .put(&[], &mut &engine_code[..], &hash_algorithm)
        .await
        .context("failed to upload engine code")?;
    let manifest = Manifest {
        version: 1,
        code_hash,
        hash_algorithm,
        args: instance_args,
        env_vars: std::env::vars()
            .map(|(key, value)| StringPair { key, value })
            .collect(),
        on_demand: false,
        resizable: false,
        max_query_size: args.query_size.try_into().context("invalid query size")?,
        label: "test".to_string(),
    };
    let app_info = worker
        .deploy_app(manifest, false)
        .await
        .context("failed to deploy the app")?;
    info!(
        "app deployed at address: 0x{:?}",
        hex_fmt::HexFmt(app_info.address)
    );
    use rocket::yansi::Paint;
    let url = format!(
        "http://localhost:{}/app/0x{}/",
        args.port,
        hex_fmt::HexFmt(app_info.address)
    );
    info!("app endpoint: {}", url.green());
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
