use anyhow::Result;

mod module;

fn default_engine() -> wasmi::Engine {
    use std::sync::OnceLock;
    static INIT: OnceLock<wasmi::Engine> = OnceLock::new();
    INIT.get_or_init(|| wasmi::Engine::default()).clone()
}

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    let wasm_ns = ns.context()?.new_object("WebAssembly");
    ns.set_property("WebAssembly", &wasm_ns)?;
    Ok(())
}
