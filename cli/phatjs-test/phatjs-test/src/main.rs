use drink::session::Session;
use pink_drink::PinkRuntime;
use phat_js as js;
use scale::{Decode, Encode};

struct Args {
    gas_limit: u64,
    output_json: bool,
    driver: String,
    script: String,
    script_args: Vec<String>,
}

fn parse_args() -> Args {
    let mut args = std::env::args().skip(1);
    let mut gas_limit = u64::MAX;
    let mut script = None;
    let mut script_args = Vec::new();
    let mut output_json = false;
    let mut driver = None;
    while let Some(arg) = args.next() {
        if !arg.starts_with("-") {
            if script.is_none() {
                script = Some(std::fs::read_to_string(&arg).unwrap_or_else(|err| {
                    println!("Failed to read script: {err}");
                    print_usage();
                }));
            }
            script_args = args.collect();
            break;
        }
        if arg == "-h" || arg == "--help" {
            print_usage();
        }
        match arg.as_str() {
            "-l" | "--gas-limit" => {
                gas_limit = args
                    .next()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or_else(|| {
                        println!("Invalid gas limit");
                        print_usage();
                    });
            }
            "-v" | "--version" => {
                println!("phatjs-test {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "-c" => {
                let code = args.next().unwrap_or_else(|| {
                    println!("Missing code");
                    print_usage();
                });
                script = Some(code);
            }
            "-j" | "--json" => {
                output_json = true;
            }
            "-1" => {
                driver = Some("JsDelegate".into());
            }
            "-2" => {
                driver = Some("JsDelegate2".into());
            }
            "-3" => {
                driver = Some("AsyncJsRuntime".into());
            }
            "--driver" => {
                let driver_name = args.next().unwrap_or_else(|| {
                    println!("Missing driver");
                    print_usage();
                });
                driver = Some(driver_name);
            }
            _ => {
                println!("Unknown option: {}", arg);
                print_usage();
            }
        }
    }
    match script {
        None => print_usage(),
        Some(script) => Args {
            driver: driver.unwrap_or_else(|| "JsDelegate".into()),
            gas_limit,
            script,
            script_args,
            output_json,
        },
    }
}

fn print_usage() -> ! {
    println!("Usage: phatjs-test [OPTIONS] <script> [script args...]");
    println!();
    println!("Options:");
    println!("  -h, --help         Print this help message");
    println!("  -v, --version      Print version info");
    println!("  -l, --gas-limit    Set gas limit");
    println!("  -j, --json         Output JSON");
    println!("  --driver <driver>  Set driver. Available drivers: JsDelegate, JsDelegate2, AsyncJsRuntime. Default: JsDelegate");
    println!("  -1                 Alias for --driver JsDelegate");
    println!("  -2                 Alias for --driver JsDelegate2");
    println!("  -3                 Alias for --driver AsyncJsRuntime");
    println!("  -c <code>          Execute code directly");
    std::process::exit(1);
}

const GAS_PER_SECOND: u64 = 1_000_000_000_000;
const QUERY_GAS_LIMIT: u64 = GAS_PER_SECOND * 10;
const TX_GAS_LIMIT: u64 = GAS_PER_SECOND / 2;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum JsValue {
    Undefined,
    Null,
    String(String),
    Bytes(String),
    Other(String),
    Exception(String),
}

impl From<js::JsValue> for JsValue {
    fn from(value: js::JsValue) -> Self {
        match value {
            js::JsValue::Undefined => Self::Undefined,
            js::JsValue::Null => Self::Null,
            js::JsValue::String(s) => Self::String(s),
            js::JsValue::Bytes(b) => Self::Bytes(hex(&b)),
            js::JsValue::Other(v) => Self::Other(v),
            js::JsValue::Exception(v) => Self::Exception(v),
        }
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Output {
    contract_exec_result: String,
    gas_consumed: u64,
    gas_required: u64,
    query_gas_limit: u64,
    tx_gas_limit: u64,
    wall_time_us: u64,
    js_output: Result<JsValue, String>,
}

fn main() {
    use pink_drink::{Callable, DeployBundle};
    use ink::codegen::TraitCallBuilder;

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let args = parse_args();

    let wasm = include_bytes!("../res/play.wasm");
    let mut session = Session::<PinkRuntime>::new().unwrap();
    let mut proxy = play::PlayRef::new()
        .deploy_wasm(wasm, &mut session)
        .unwrap();

    let start = std::time::Instant::now();
    let result = proxy
        .call_mut()
        .eval_js(args.driver, args.script, args.script_args)
        .gas_limit(args.gas_limit)
        .bare_query(&mut session);
    let elapsed = start.elapsed();
    tracing::info!("ExecResult: {:?}", result);
    let mut js_output = Err("No output".to_string());
    if let Ok(result) = &result.result {
        let result: Result<Result<js::JsValue, String>, u8> =
            Decode::decode(&mut &result.data[..]).expect("Failed to decode result");
        match result {
            Ok(Ok(output)) => {
                js_output = Ok(output.into());
            }
            Ok(Err(e)) => {
                js_output = Err(e);
            }
            Err(e) => {
                js_output = Err(format!("ink::LangError({e})"));
            }
        }
        if !args.output_json {
            println!();
            println!("JS output: {js_output:?}");
        }
    }
    fn percent(a: u64, b: u64) -> f64 {
        a as f64 / b as f64 * 100.0
    }
    let gas_required = result.gas_required.ref_time();
    let gas_consumed = result.gas_consumed.ref_time();
    let gas_consumed_time = gas_consumed as f64 / GAS_PER_SECOND as f64;
    let time_scaled = gas_consumed_time / elapsed.as_secs_f64();
    if args.output_json {
        let output = Output {
            contract_exec_result: hex(&result.encode()),
            gas_consumed,
            gas_required,
            query_gas_limit: QUERY_GAS_LIMIT,
            tx_gas_limit: TX_GAS_LIMIT,
            wall_time_us: elapsed.as_micros() as u64,
            js_output,
        };
        println!();
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        println!();
        println!("==================== Gas usage ====================");
        println!("Wall time         : {:.3}s", elapsed.as_secs_f64());
        println!(
            "Gas consumed      : {} ({:.3}s)",
            gas_consumed, gas_consumed_time
        );
        println!("Gas required      : {}", gas_required);
        println!("Max gas for query : {}", QUERY_GAS_LIMIT);
        println!(
            "Gas required/query: {:.2}%",
            percent(gas_required, QUERY_GAS_LIMIT)
        );
        println!("Max gas for tx    : {}", TX_GAS_LIMIT);
        println!(
            "Gas required/tx   : {:.2}%",
            percent(gas_required, TX_GAS_LIMIT)
        );
        println!("Time scaled       : {:.2}x", time_scaled);
        println!("===================================================");
    }
}

fn hex(hex: &[u8]) -> String {
    format!("0x{}", hex_fmt::HexFmt(hex))
}

#[ink::contract]
mod play {
    use phat_js::JsValue;

    #[ink(storage)]
    pub struct Play {}

    impl Play {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn eval_js(
            &self,
            _driver: String,
            _script: String,
            _args: Vec<String>,
        ) -> Result<JsValue, String> {
            unimplemented!()
        }
    }
}
