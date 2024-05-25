use anyhow::Result;
use js::AsBytes;

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("workerSign", worker_sign)?;
    ns.define_property_fn("workerPublicKey", worker_pubkey)?;
    ns.define_property_fn("sgxQuote", sgx_quote)?;
    Ok(())
}

#[js::host_call]
fn worker_sign(message: js::BytesOrString) -> Result<AsBytes<Vec<u8>>> {
    wapo::ocall::sign(message.as_bytes())
        .map(AsBytes)
        .map_err(Into::into)
}

#[js::host_call]
fn worker_pubkey() -> Result<AsBytes<Vec<u8>>> {
    wapo::ocall::worker_pubkey()
        .map(AsBytes)
        .map_err(Into::into)
}

#[js::host_call]
fn sgx_quote(message: js::BytesOrString) -> Result<Option<AsBytes<Vec<u8>>>> {
    let quote = wapo::ocall::sgx_quote(message.as_bytes())?;
    Ok(quote.map(AsBytes))
}
