use anyhow::Result;
use js::IntoJsValue;

pub fn setup(ns: &js::Value) -> Result<()> {
    let ctx = ns.context()?;
    let env = ctx.new_object();
    for (key, value) in std::env::vars() {
        env.set_property(&key, &value.into_js_value(ctx)?)?;
    }
    ns.set_property("env", &env)?;
    Ok(())
}
