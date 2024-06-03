use anyhow::Result;

mod engine;
mod global;
mod module;

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    let wasm_ns = ns.context()?.new_object("WebAssembly");
    ns.set_property("WebAssembly", &wasm_ns)?;
    global::setup(&wasm_ns)?;
    Ok(())
}
