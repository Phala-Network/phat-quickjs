#[cfg(test)]
mod tests {
    use ink::codegen::TraitCallBuilder;

    use drink::session::Session;
    use drink_pink_runtime::{Callable, DeployBundle, PinkRuntime};

    use control::ControlRef;

    #[drink::contract_bundle_provider]
    enum BundleProvider {}

    #[test]
    fn bench_js_delegate() -> Result<(), Box<dyn std::error::Error>> {
        tracing_subscriber::fmt::init();
        let mut session = Session::<PinkRuntime>::new()?;
        let control = ControlRef::default()
            .deploy_bundle(&BundleProvider::Control.bundle()?, &mut session)?;
        let bench_src = include_str!("../../../examples/scale-bench.js");
        _ = control
            .call()
            .run_js_using_delegate(bench_src.to_string(), vec![])
            .query(&mut session)?;
        _ = control
            .call()
            .run_js_using_delegate2(bench_src.to_string(), vec![])
            .query(&mut session)?;
        Ok(())
    }
}
