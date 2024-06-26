use anyhow::{bail, Context, Result};
use js::{AsBytes, BytesOrString, FromJsValue, JsUint8Array, ToJsValue};
use riscvm::{ExecutorEnv, ExecutorImpl, ExitCode};

use super::http_request::Pairs;

#[derive(FromJsValue, Debug)]
#[qjs(rename_all = "camelCase")]
pub struct ExecRequest {
    program: JsUint8Array,
    #[qjs(default)]
    args: Vec<String>,
    #[qjs(default)]
    env: Pairs,
    #[qjs(default)]
    stdin: BytesOrString,
}

#[derive(ToJsValue, Debug)]
#[qjs(rename_all = "camelCase")]
struct ExecResponse {
    compute_id: AsBytes<Vec<u8>>,
    exit_code: u32,
    journal: AsBytes<Vec<u8>>,
}

pub fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("unstable_runRisc0Program", riscv_run_elf)?;
    Ok(())
}

#[js::host_call]
fn riscv_run_elf(req: ExecRequest) -> Result<ExecResponse> {
    let mut builder = ExecutorEnv::builder();
    for (k, v) in req.env.pairs.into_iter() {
        builder.env_var(&k, &v);
    }
    let env = builder
        .args(&req.args)
        .stdin(req.stdin.as_bytes())
        .build()?;
    let mut instance = ExecutorImpl::from_elf(env, req.program.as_bytes())
        .context("Failed to load riscv program")?;
    let session = instance.run().context("Failed to run riscv program")?;
    let exit_code = match session.exit_code {
        ExitCode::Halted(code) => code,
        ExitCode::SessionLimit => bail!("Session limit reached"),
    };
    Ok(ExecResponse {
        compute_id: AsBytes(instance.compute_id().as_bytes().to_vec()),
        exit_code,
        journal: AsBytes(session.journal.unwrap_or_default().bytes),
    })
}
