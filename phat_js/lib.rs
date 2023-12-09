//! A library for interacting with the contract phat-quickjs
//!
//! The available JS APIs can be found [here](https://github.com/Phala-Network/phat-quickjs/blob/master/npm_package/pink-env/src/index.ts).
//!
//! # Script args and return value
//!
//! The `eval_*` functions take a script as source code and args as input. It eval the source by delegate call to the code pointed to by
//! driver `JsDelegate2` and return the value of the js expression. The return value only lost-less supports string or Uint8Array. Ojbects
//! of other types will be casted to string.
//!
//! Example:
//! ```no_run
//! let src = r#"
//! (function () {
//!     return scriptArgs[0] + scriptArgs[1];
//! })()
//! "#;
//! let output = phat_js::eval(src, &["foo".into(), "bar".into()]);
//! assert_eq!(output, Ok(phat_js::Output::String("foobar".into())));
//! ```
//!
//! # JS API examples
//!
//! ## Cross-contract call
//!
//! ```js
//! // Delegate calling
//! const delegateOutput = pink.invokeContractDelegate({
//!     codeHash:
//!       "0x0000000000000000000000000000000000000000000000000000000000000000",
//!     selector: 0xdeadbeef,
//!     input: "0x00",
//!   });
//!   
//!   // Instance calling
//!  const contractOutput = pink.invokeContract({
//!    callee: "0x0000000000000000000000000000000000000000000000000000000000000000",
//!    input: "0x00",
//!    selector: 0xdeadbeef,
//!    gasLimit: 0n,
//!    value: 0n,
//!  });
//! ```
//!
//! This is the low-level API for cross-contract call.
//! If you have the contract metadata file, there is a [script](https://github.com/Phala-Network/phat-quickjs/blob/master/scripts/meta2js.py) helps
//! to generate the high-level API for cross-contract call. For example run the following command:
//!
//! ```shell
//! python meta2js.py --keep System::version /path/to/system.contract
//! ```
//! would generate the following code:
//! ```js
//! function invokeContract(callee, selector, args, metadata, registry) {
//!     const inputCodec = pink.SCALE.codec(metadata.inputs, registry);
//!     const outputCodec = pink.SCALE.codec(metadata.output, registry);
//!     const input = inputCodec.encode(args ?? []);
//!     const output = pink.invokeContract({ callee, selector, input });
//!     return outputCodec.decode(output);
//! }
//! class System {
//!     constructor(address) {
//!         this.typeRegistryRaw = '#u16\n(0,0,0)\n<CouldNotReadInput::1>\n<Ok:1,Err:2>'
//!         this.typeRegistry = pink.SCALE.parseTypes(this.typeRegistryRaw);
//!         this.address = address;
//!     }
//!   
//!     system$Version() {
//!         const io = {"inputs": [], "output": 3};
//!         return invokeContract(this.address, 2278132365, [], io, this.typeRegistry);
//!     }
//! }
//! ```
//!
//! Then you can use the high-level API to call the contract:
//! ```js
//! const system = new System(systemAddress);
//! const version = system.system$Version();
//! console.log("version:", version);
//! ```
//!
//! ## HTTP request
//!
//! HTTP request is supported in the JS environment. However, the API is sync rather than async.
//! This is different from other JavaScript engines. For example:
//! ```js
//! const response = pink.httpReqeust({
//!   url: "https://httpbin.org/ip",
//!   method: "GET",
//!   returnTextBody: true,
//! });
//! console.log(response.body);
//! ```
//!
//! ## SCALE codec
//!
//! Let's introduce the details of the SCALE codec API which is not documented in the above link.
//!
//! The SCALE codec API is mounted on the global object `pink.SCALE` which contains the following functions:
//!
//! - `pink.SCALE.parseTypes(types: string): TypeRegistry`
//! - `pink.SCALE.codec(type: string | number | number[], typeRegistry?: TypeRegistry): Codec`
//!
//! Let's make a basice example to show how to use the SCALE codec API:
//!
//! ```js
//! const types = `
//!   Hash=[u8;32]
//!   Info={hash:Hash,size:u32}
//! `;
//! const typeRegistry = pink.SCALE.parseTypes(types);
//! const infoCodec = pink.SCALE.codec(`Info`, typeRegistry);
//! const encoded = infoCodec.encode({
//!  hash: "0x1234567890123456789012345678901234567890123456789012345678901234",
//!  size: 1234,
//! });
//! console.log("encoded:", encoded);
//! const decoded = infoCodec.decode(encoded);
//! pink.inspect("decoded:", decoded);
//! ```
//!
//! The above code will output:
//! ```text
//! JS: encoded: 18,52,86,120,144,18,52,86,120,144,18,52,86,120,144,18,52,86,120,144,18,52,86,120,144,18,52,86,120,144,18,52,210,4,0,0
//! JS: decoded: {
//! JS: hash: 0x1234567890123456789012345678901234567890123456789012345678901234,
//! JS: size: 1234
//! JS: }
//! ```
//!
//! Or using the direct encode/decode api which support literal type definition as well as a typename or id, for example:
//!
//! ```js
//! const data = { name: "Alice", age: 18 };
//! const encoded = pink.SCALE.encode(data, "{ name: str, age: u8 }");
//! const decoded = pink.SCALE.decode(encoded, "{ name: str, age: u8 }");
//! ```
//!
//! ## Grammar of the type definition
//! ### Basic grammar
//! In the above example, we use the following type definition:
//! ```text
//! Hash=[u8;32]
//! Info={hash:Hash,size:u32}
//! ```
//! where we define a type `Hash` which is an array of 32 bytes, and a type `Info` which is a struct containing a `Hash` and a `u32`.
//!
//! The grammar is defined as follows:
//!
//! Each entry is type definition, which is of the form `name=type`. Where name must be a valid identifier,
//! and type is a valid type expression described below.
//!
//! Type expression can be one of the following:
//!
//! | Type Expression          | Description                               | Example          | JS type |
//! |-------------------------|-------------------------------------------|------------------|--------------------|
//! | `bool` | Primitive type bool |  | `true`, `false` |
//! | `u8`, `u16`, `u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128` | Primitive number types |   | number or bigint |
//! | `str` | Primitive type str |   | string |
//! | `[type;size]`           | Array type with element type `type` and size `size`. | `[u8; 32]` | Array of elements. (Uint8Array or `0x` prefixed hex string is allowed for [u8; N]) |
//! | `[type]`                | Sequence type with element type `type`. | `[u8]` | Array of elements. (Uint8Array or `0x` prefixed hex string is allowed for [u8]) |
//! | `(type1, type2, ...)`   | Tuple type with elements of type `type1`, `type2`, ... | `(u8, str)` | Array of value for inner type. (e.g. `[42, 'foobar']`) |
//! | `{field1:type1, field2:type2, ...}` | Struct type with fields and types. | `{age:u32, name:str}` | Object with field name as key |
//! | `<variant1:type1, variant2:type2, ...>` | Enum type with variants and types. if the variant is a unit variant, then the type expression can be omitted.| `<Success:i32, Error:str>`, `<None,Some:u32>` | Object with variant name as key. (e.g. `{Some: 42}`)|
//! | `@type` | Compact number types. Only unsigned number types is supported | `@u64` | number or bigint |
//!
//! ### Generic type support
//!
//! Generic parameters can be added to the type definition, for example:
//!
//! ```text
//! Vec<T>=[T]
//! ```
//!
//! ### Option type
//! The Option type is not a special type, but a vanilla enum type. It is needed to be defined by the user explicitly. Same for the Result type.
//!
//! ```text
//! Option<T>=<None,Some:T>
//! Result<T,E>=<Ok:T,Err:E>
//! ```
//!
//! There is one special syntax for the Option type:
//! ```text
//! Option<T>=<_None,_Some:T>
//! ```
//! If the Option type is defined in this way, then the `None` variant would be decoded as `null` instead of `{None: null}` and the `Some` variant would be decoded as the inner value directly instead of `{Some: innerValue}`.
//! For example:
//! ```js
//! const encoded = pink.SCALE.encode(42, "<_None,_Some:u32>");
//! const decoded = pink.SCALE.decode(encoded, "<_None,_Some:u32>");
//! console.log(decoded); // 42
//! ```
//!
//! ### Nested type definition
//!
//! Type definition can be nested, for example:
//!
//! ```text
//! Block={header:{hash:[u8;32],size:u32}}
//! ```
//!
//! ## Error handling
//! Host calls would throw an exception if any error is encountered. For example, if we pass an invalid method to the API:
//! ```js
//!  try {
//!    const response = pink.httpReqeust({
//!      url: "https://httpbin.org/ip",
//!      method: 42,
//!      returnTextBody: true,
//!    });
//!    console.log(response.body);
//!  } catch (err) {
//!    console.log("Some error ocurred:", err);
//!  }
//! ```
//!
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use ink::env::call;
use ink::primitives::Hash;
use scale::{Decode, Encode};

pub use pink::chain_extension::{JsCode, JsValue};

#[derive(Debug, Encode, Decode, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum GenericValue<S, B> {
    String(S),
    Bytes(B),
    Undefined,
}
pub type RefValue<'a> = GenericValue<&'a str, &'a [u8]>;
pub type Value = GenericValue<String, Vec<u8>>;
pub type Output = Value;

impl From<Value> for JsValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Undefined => Self::Undefined,
            Value::String(v) => Self::String(v),
            Value::Bytes(v) => Self::Bytes(v),
        }
    }
}

pub fn default_delegate() -> Result<Hash, String> {
    let system = pink::system::SystemRef::instance();
    let delegate = system
        .get_driver("JsDelegate2".into())
        .ok_or("No JS driver found")?;
    Ok(delegate.convert_to())
}

/// Evaluate a script with the default delegate contract code
pub fn eval(script: &str, args: &[String]) -> Result<Output, String> {
    eval_with(default_delegate()?, script, args)
}

/// Evaluate a compiled bytecode with the default delegate contract code
pub fn eval_bytecode(code: &[u8], args: &[String]) -> Result<Output, String> {
    eval_bytecode_with(default_delegate()?, code, args)
}

/// Evaluate multiple scripts with the default delegate contract code
pub fn eval_all(codes: &[RefValue], args: &[String]) -> Result<Output, String> {
    eval_all_with(default_delegate()?, codes, args)
}

/// Evaluate a script with given delegate contract code
pub fn eval_with(delegate: Hash, script: &str, args: &[String]) -> Result<Output, String> {
    call::build_call::<pink::PinkEnvironment>()
        .call_type(call::DelegateCall::new(delegate))
        .exec_input(
            call::ExecutionInput::new(call::Selector::new(ink::selector_bytes!("eval")))
                .push_arg(script)
                .push_arg(args),
        )
        .returns::<Result<Output, String>>()
        .invoke()
}

/// Evaluate a compiled script with given delegate contract code
pub fn eval_bytecode_with(
    delegate: Hash,
    script: &[u8],
    args: &[String],
) -> Result<Output, String> {
    call::build_call::<pink::PinkEnvironment>()
        .call_type(call::DelegateCall::new(delegate))
        .exec_input(
            call::ExecutionInput::new(call::Selector::new(ink::selector_bytes!("eval_bytecode")))
                .push_arg(script)
                .push_arg(args),
        )
        .returns::<Result<Output, String>>()
        .invoke()
}

/// Evaluate multiple scripts with given delegate
pub fn eval_all_with(
    delegate: Hash,
    scripts: &[RefValue],
    args: &[String],
) -> Result<Output, String> {
    call::build_call::<pink::PinkEnvironment>()
        .call_type(call::DelegateCall::new(delegate))
        .exec_input(
            call::ExecutionInput::new(call::Selector::new(ink::selector_bytes!("eval_all")))
                .push_arg(scripts)
                .push_arg(args),
        )
        .returns::<Result<Output, String>>()
        .invoke()
}

/// Evaluate async JavaScript with SideVM QuickJS.
///
/// This function is similar to [`eval`], but it uses SideVM QuickJS to evaluate the script.
/// This function would polyfill `pink.SCALE`, `pink.hash`, `pink.deriveSecret` which are not
/// available in SideVM QuickJS.
///
/// # Parameters
///
/// * `code`: A JavaScript code that can be either a source code or bytecode.
/// * `args`: A vector of strings that contain arguments passed to the JavaScript code.
///
/// # Returns
///
/// * a `JsValue` object which represents the evaluated result of the JavaScript code.
///
/// # Examples
///
/// ```ignore
/// let js_code = phat_js::JsCode::Source("setTimeout(() => { scriptOutput = '42'; }, 100);".into());
/// let res = phat_js::eval_async_js(js_code, Vec::new());
/// assert_eq!(res, JsValue::String("42".into()));
/// ```
pub fn eval_async_js(code: JsCode, args: Vec<String>) -> JsValue {
    let code_bytes = match &code {
        JsCode::Source(source) => source.as_bytes(),
        JsCode::Bytecode(bytecode) => bytecode.as_slice(),
    };
    let mut code_hash = Default::default();
    ink::env::hash_bytes::<ink::env::hash::Blake2x256>(code_bytes, &mut code_hash);
    let polyfill = polyfill_script(pink::vrf(&code_hash));
    let codes = alloc::vec![JsCode::Source(polyfill), code];
    pink::ext().js_eval(codes, args)
}

fn polyfill_script(seed: impl AsRef<[u8]>) -> String {
    let seed = hex_fmt::HexFmt(seed);
    alloc::format!(
        r#"
        (function(g) {{
            const seed = "{seed}";
            const {{ SCALE, hash, hexEncode }} = Sidevm;
            g.pink = g.Pink = {{
                SCALE,
                hash,
                deriveSecret(salt) {{
                    return hash('blake2b512', seed + hexEncode(salt));
                }},
                vrf(salt) {{
                    return hash('blake2b512', 'vrf:' + seed + hexEncode(salt));
                }}
            }};
        }})(globalThis);
    "#
    )
}

/// Compile a script with the default delegate contract
pub fn compile(script: &str) -> Result<Vec<u8>, String> {
    compile_with(default_delegate()?, script)
}

/// Compile a script with given delegate contract
pub fn compile_with(delegate: Hash, script: &str) -> Result<Vec<u8>, String> {
    call::build_call::<pink::PinkEnvironment>()
        .call_type(call::DelegateCall::new(delegate))
        .exec_input(
            call::ExecutionInput::new(call::Selector::new(ink::selector_bytes!("compile")))
                .push_arg(script),
        )
        .returns::<Result<Vec<u8>, String>>()
        .invoke()
}

pub trait ConvertTo<To> {
    fn convert_to(&self) -> To;
}

impl<F, T> ConvertTo<T> for F
where
    F: AsRef<[u8; 32]>,
    T: From<[u8; 32]>,
{
    fn convert_to(&self) -> T {
        (*self.as_ref()).into()
    }
}
