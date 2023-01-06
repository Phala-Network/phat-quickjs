use alloc::vec::Vec;
use ink_env::{call, Result};
use pink_extension::PinkEnvironment;
use scale::{Decode, Encode};

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
    delegate: ink_env::Hash,
    selector: u32,
    input: &[u8],
) -> Result<Vec<u8>> {
    call::build_call::<PinkEnvironment>()
        .call_type(call::DelegateCall::new().code_hash(delegate))
        .exec_input(
            call::ExecutionInput::new(call::Selector::new(selector.to_be_bytes()))
                .push_arg(RawBytes(input)),
        )
        .returns::<RawBytes<Vec<u8>>>()
        .fire()
        .map(|x| x.0)
}

pub(crate) fn invoke_contract(
    callee: ink_env::AccountId,
    gas_limit: u64,
    transferred_value: u128,
    selector: u32,
    input: &[u8],
) -> Result<Vec<u8>> {
    let call_type = call::Call::new()
        .callee(callee)
        .gas_limit(gas_limit)
        .transferred_value(transferred_value);
    call::build_call::<PinkEnvironment>()
        .call_type(call_type)
        .exec_input(
            call::ExecutionInput::new(call::Selector::new(selector.to_be_bytes()))
                .push_arg(RawBytes(input)),
        )
        .returns::<RawBytes<Vec<u8>>>()
        .fire()
        .map(|x| x.0)
}
