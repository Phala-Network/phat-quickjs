use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use wapod::{
    config::{AddressGenerator, DefaultKerProvider, Paths, WorkerConfig},
    types::ticket::AppManifest,
    Address,
};

pub struct Config;

impl WorkerConfig for Config {
    type AddressGenerator = Self;
    type KeyProvider = DefaultKerProvider<Self>;
    type Paths = Self;
}

impl AddressGenerator for Config {
    fn generate_address(_manifest: &AppManifest) -> Address {
        [0; 32]
    }
}

impl Paths for Config {
    fn data_dir() -> PathBuf {
        config_dir().join("data")
    }
}

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .map(|p| p.join("wapojs"))
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| "/".into())
                .join(".wapojs")
        })
}

fn engine_config_file() -> PathBuf {
    config_dir().join("default_engine")
}

pub fn read_default_engine() -> Option<String> {
    std::fs::read_to_string(engine_config_file()).ok()
}

pub fn save_default_engine(engine: &Path) -> Result<()> {
    let path = engine_config_file();
    std::fs::create_dir_all(path.parent().context("no parent")?).context("failed to create dir")?;
    std::fs::write(path, engine.to_str().context("non string path")?.as_bytes())
        .context("failed to write engine code")
}
