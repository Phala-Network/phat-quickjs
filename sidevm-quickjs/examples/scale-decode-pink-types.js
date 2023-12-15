#!/usr/bin/env phatjs
const R = globalThis.Sidevm || globalThis.Pink;
const { repr, inspect, hexDecode, SCALE: scl } = R;
console.log = inspect;

const typeRegistry = `
Option<T>=<_None,_Some:T>
Result<T,E>=<Ok:T,Err:E>
Vec<T>=[T]
Hash=[u8;32]
Phase=<
	/// Applying an extrinsic.
	ApplyExtrinsic:u32,
	/// Finalizing the block.
	Finalization,
	/// Initializing the block.
	Initialization,
>
EventRecord<E,T>={
	/// The phase of the block it happened in.
	phase: Phase,
	/// The event itself.
	event: E,
	/// The list of the topics this event has.
	topics: Vec<T>,
}
PinkEventRecord=EventRecord<RuntimeEvent,Hash>
Weight={
	ref_time: @u64,
	proof_size: @u64,
}
ContractExecResult=ContractResult<Result<ExecReturnValue,DispatchError>,u128>
ContractResult<R, Balance>={
	gas_consumed: Weight,
	gas_required: Weight,
	storage_deposit: StorageDeposit<Balance>,
	debug_message: Vec<u8>,
	result: R,
}
StorageDeposit<Balance>=<
	Refund:Balance,
	Charge:Balance,
>
ExecReturnValue={
	flags: ReturnFlags,
	data: Vec<u8>,
}
ReturnFlags=u32
DispatchError=<
	Other,
	CannotLookup,
	BadOrigin,
	Module:ModuleError,
	ConsumerRemaining,
	NoProviders,
	TooManyConsumers,
	Token:TokenError,
	Arithmetic:ArithmeticError,
	Transactional:TransactionalError,
	Exhausted,
	Corruption,
	Unavailable,
	RootNotAllowed,
>
ModuleError={
	index: u8,
	error: [u8; 4],
}
TokenError=<
	FundsUnavailable,
	OnlyProvider,
	BelowMinimum,
	CannotCreate,
	UnknownAsset,
	Frozen,
	Unsupported,
	CannotCreateHold,
	NotExpendable,
	Blocked,
>
ArithmeticError=<
	Underflow,
	Overflow,
	DivisionByZero,
>
TransactionalError=<
	LimitReached,
	NoLayer,
>
ContractError=<
    InvalidScheduleVersion,
    InvalidCallFlags,
    OutOfGas,
    OutputBufferTooSmall,
    TransferFailed,
    MaxCallDepthReached,
    ContractNotFound,
    CodeTooLarge,
    CodeNotFound,
    OutOfBounds,
    DecodingFailed,
    ContractTrapped,
    ValueTooLarge,
    TerminatedWhileReentrant,
    InputForwarded,
    RandomSubjectTooLong,
    TooManyTopics,
    NoChainExtension,
    DuplicateContract,
    TerminatedInConstructor,
    ReentranceDenied,
    StorageDepositNotEnoughFunds,
    StorageDepositLimitExhausted,
    CodeInUse,
    ContractReverted,
    CodeRejected,
    Indeterministic,
>
InkCommand=<
    InkMessage: {
        nonce: [u8],
        message: [u8],
        transfer: u128,
        gasLimit: u64,
        storageDepositLimit: Option<u128>,
    },
>
AeadIV = [u8; 12]
EcdhPublicKey = [u8; 32]
EncryptedData = {
    iv: AeadIV,
    pubkey: EcdhPublicKey,
    data: Vec<u8>,
}
Payload<T> = <Plain:T|Encrypted:EncryptedData>
Message = Payload<InkCommand>
`

const results = [
    ["2023-09-14T01:33:00+08:00	blockNumber=3408094", "0x000000000100000000000000000000000000000000000103041500000000"],
    ["2023-09-13T19:45:29+08:00	blockNumber=3406371", "0x03909741e88ee50500070000807e1202008000010000000000000000000000000000000000000000000008000000"],
    ["2023-09-13T19:43:56+08:00	blockNumber=3406363", "0x00000000010000000000000000000000000000000000010000"],
    ["2023-09-13T19:42:59+08:00	blockNumber=3406359", "0x03d55214e7c2d80500070000807e12020080000100660b69480000000000000000000000000000000000280000020000000000000000"],
    ["2023-09-13T00:55:23+08:00	blockNumber=3400765", "0x03909741e88ee50500070000807e1202008000010000000000000000000000000000000000000000000008000000"],
    ["2023-09-12T14:27:01+08:00	blockNumber=3397658", "0x00000000010000000000000000000000000000000000010000"],
    ["2023-09-12T14:26:18+08:00	blockNumber=3397653", "0x03d55214e7c2d80500070000807e12020080000100660b69480000000000000000000000000000000000280000010000000000000000"],
]

for (let i = 0; i < results.length; i++) {
    const [time, contractOutput] = results[i];
    const decoded = scl.decode(hexDecode(contractOutput), 'ContractExecResult', typeRegistry);
    console.log(time, repr(decoded.result));
    const moduleError = decoded.result.Err?.Module;
    if (moduleError && moduleError.index == 4) {
        const contractError = scl.decode(moduleError.error, 'ContractError', typeRegistry);
        console.log('ContractError:', contractError);
    }
}
