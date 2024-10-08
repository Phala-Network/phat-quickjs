use anyhow::Result;

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    #[cfg(feature = "wapo")]
    ns.define_property_fn("deriveSecret", derive_secret)?;
    #[cfg(not(feature = "wapo"))]
    let _ = ns;
    Ok(())
}

#[cfg(feature = "wapo")]
#[js::host_call]
fn derive_secret(path: js::BytesOrString) -> Result<js::AsBytes<[u8; 64]>> {
    wapo::ocall::derive_secret(path.as_bytes())
        .map(js::AsBytes)
        .map_err(Into::into)
}
