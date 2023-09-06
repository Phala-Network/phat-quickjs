#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use ink::env::call;
use ink::primitives::Hash;
use scale::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum GenericValue<S, B> {
    String(S),
    Bytes(B),
    Undefined,
}
pub type RefValue<'a> = GenericValue<&'a str, &'a [u8]>;
pub type Value = GenericValue<String, Vec<u8>>;
pub type Output = Value;

fn js_delegate() -> Result<Hash, String> {
    let system = pink::system::SystemRef::instance();
    let delegate = system
        .get_driver("JsDelegate".into())
        .ok_or("No JS driver found")?;
    Ok(delegate.convert_to())
}

/// Evaluate a script with the default delegate contract code
pub fn eval(script: &str, args: &[String]) -> Result<Output, String> {
    eval_with(js_delegate()?, script, args)
}

/// Evaluate a compiled bytecode with the default delegate contract code
pub fn eval_bytecode(code: &[u8], args: &[String]) -> Result<Output, String> {
    eval_bytecode_with(js_delegate()?, code, args)
}

/// Evaluate multiple scripts with the default delegate contract code
pub fn eval_all(codes: &[RefValue], args: &[String]) -> Result<Output, String> {
    eval_all_with(js_delegate()?, codes, args)
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
