use anyhow::Result;
use js::FromJsContext;

mod engine;
mod global;
mod instance;
mod memory;
mod module;

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    let ctx = ns.context()?;
    let wasm_ns = ctx.new_object("WebAssembly");
    ns.set_property("WebAssembly", &wasm_ns)?;
    ctx.eval(&js::Code::Bytecode(qjsc::compiled!(
        r#"
        WebAssembly.compile = async function (source) {
            return new WebAssembly.Module(new Uint8Array(source));
        };
        WebAssembly.compileStreaming = async function (source) {
            let wasm = await source;
            if (wasm instanceof Response) {
                if (!wasm.ok) {
                    throw new Error("failed to fetch wasm, status: " + wasm.status);
                }
                wasm = await wasm.arrayBuffer();
            }
            return WebAssembly.compile(wasm);
        };
        WebAssembly.instantiate = async function (moduleOrBytes, imports) {
            if (moduleOrBytes instanceof WebAssembly.Module) {
                return new WebAssembly.Instance(moduleOrBytes, imports);
            } else {
                const module = await WebAssembly.compile(moduleOrBytes);
                const instance = await WebAssembly.instantiate(module, imports);
                return { module, instance };
            }
        };
        WebAssembly.instantiateStreaming = async function (source, imports) {
            const module = await WebAssembly.compileStreaming(source);
            const instance = await WebAssembly.instantiate(module, imports);
            return { module, instance };
        };
    "#
    )))
    .map_err(js::Error::msg)?;

    module::setup(&wasm_ns)?;
    global::setup(&wasm_ns)?;
    memory::setup(&wasm_ns)?;
    instance::setup(&wasm_ns)?;

    wasm_ns.define_property_fn("validate", validate)?;
    wasm_ns.define_property_fn("parseWat", parse_wat)?;
    Ok(())
}

#[js::host_call(with_context)]
fn validate(ctx: js::Context, _this: js::Value, source: js::Bytes) -> Result<bool> {
    let store = engine::GlobalStore::from_js_context(&ctx)?;
    Ok(wasmi::Module::validate(&store.engine(), source.as_bytes()).is_ok())
}

#[js::host_call]
fn parse_wat(source: js::BytesOrString) -> Result<js::Bytes> {
    let wasm = wat::parse_bytes(source.as_bytes())?;
    Ok(wasm.into_owned().into())
}
