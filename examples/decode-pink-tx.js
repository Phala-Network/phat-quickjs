#!/usr/bin/env qjs
const R = globalThis.Sidevm || globalThis.Pink;
const scl = R.SCALE;
console.log = R.inspect;

const typeRegistry = `
Option<T>=<None,Some:T>
InkCommand=<
    InkMessage: {
        nonce: [u8],
        message: [u8],
        transfer: u128,
        gasLimit: u64,
        storageDepositLimit: Option<u128>,
    },
>
`
const encodedBytes = R.hexDecode(scriptArgs[0]);
const decoded = scl.decode(encodedBytes, 'InkCommand', typeRegistry);
console.log();
console.log(decoded);
