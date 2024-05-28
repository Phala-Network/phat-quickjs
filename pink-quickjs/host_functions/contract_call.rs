use alloc::vec::Vec;
use ink::env::{call, Result};
use pink::PinkEnvironment;
use qjsbind as js;
use scale::{Decode, Encode};

pub fn setup(pink: &js::Value) -> js::Result<()> {
    pink.define_property_fn("invokeContract", contract_call)?;
    pink.define_property_fn("invokeContractDelegate", delegate_call)?;
    Ok(())
}

#[derive(Debug, js::FromJsValue)]
#[qjs(rename_all = "camelCase")]
struct ContractCall {
    #[qjs(bytes_or_hex)]
    callee: [u8; 32],
    #[qjs(default)]
    gas_limit: u64,
    #[qjs(default)]
    value: u128,
    selector: u32,
    #[qjs(default, bytes_or_hex)]
    input: Vec<u8>,
    #[qjs(default)]
    allow_reentry: bool,
}

#[js::host_call]
fn contract_call(call: ContractCall) -> Result<Vec<u8>> {
    invoke_contract(
        call.callee.into(),
        call.gas_limit,
        call.value,
        call.selector,
        &call.input,
        call.allow_reentry,
    )
}

#[derive(Debug, js::FromJsValue)]
#[qjs(rename_all = "camelCase")]
struct DelegateCall {
    #[qjs(bytes_or_hex)]
    code_hash: [u8; 32],
    selector: u32,
    #[qjs(default, bytes_or_hex)]
    input: Vec<u8>,
    #[qjs(default)]
    allow_reentry: bool,
}

#[js::host_call]
fn delegate_call(call: DelegateCall) -> Result<Vec<u8>> {
    invoke_contract_delegate(
        call.code_hash.into(),
        call.selector,
        &call.input,
        call.allow_reentry,
    )
}

struct RawBytes<T>(T);

impl Decode for RawBytes<Vec<u8>> {
    fn decode<I: scale::Input>(input: &mut I) -> core::result::Result<Self, scale::Error> {
        let len = input
            .remaining_len()?
            .ok_or("Can not decode RawBytes without length")?;
        let mut decoded = alloc::vec![0; len];
        input.read(&mut decoded)?;
        Ok(RawBytes(decoded))
    }
}

impl<T: AsRef<[u8]>> Encode for RawBytes<T> {
    fn size_hint(&self) -> usize {
        self.0.as_ref().len()
    }

    fn encode_to<O: scale::Output + ?Sized>(&self, dest: &mut O) {
        dest.write(self.0.as_ref());
    }

    fn encoded_size(&self) -> usize {
        self.0.as_ref().len()
    }
}

pub(crate) fn invoke_contract_delegate(
    delegate: ink::primitives::Hash,
    selector: u32,
    input: &[u8],
    allow_reentry: bool,
) -> Result<Vec<u8>> {
    let flags = ink::env::CallFlags::default().set_allow_reentry(allow_reentry);
    call::build_call::<PinkEnvironment>()
        .call_type(call::DelegateCall::new(delegate))
        .call_flags(flags)
        .exec_input(
            call::ExecutionInput::new(call::Selector::new(selector.to_be_bytes()))
                .push_arg(RawBytes(input)),
        )
        .returns::<RawBytes<Vec<u8>>>()
        .try_invoke()
        .map(|x| x.encode())
}

pub(crate) fn invoke_contract(
    callee: ink::primitives::AccountId,
    gas_limit: u64,
    transferred_value: u128,
    selector: u32,
    input: &[u8],
    allow_reentry: bool,
) -> Result<Vec<u8>> {
    let call_type = call::Call::new(callee)
        .gas_limit(gas_limit)
        .transferred_value(transferred_value);
    let flags = ink::env::CallFlags::default().set_allow_reentry(allow_reentry);
    call::build_call::<PinkEnvironment>()
        .call_type(call_type)
        .call_flags(flags)
        .exec_input(
            call::ExecutionInput::new(call::Selector::new(selector.to_be_bytes()))
                .push_arg(RawBytes(input)),
        )
        .returns::<RawBytes<Vec<u8>>>()
        .try_invoke()
        .map(|x| x.encode())
}
