use wapod::{
    config::{AddressGenerator, DefaultWorkerConfig, WorkerConfig},
    rpc::prpc::Manifest,
    Address,
};

pub struct Config;

impl WorkerConfig for Config {
    type AddressGenerator = Self;
    type KeyProvider = DefaultWorkerConfig;
    type Paths = DefaultWorkerConfig;
}

impl AddressGenerator for Config {
    fn generate_address(_manifest: &Manifest) -> Address {
        [0; 32]
    }
}
