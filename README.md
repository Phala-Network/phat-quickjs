# Phat QuickJS

A phat contract that ports the QuickJS engine to pink environment.

## Build (Ubuntu 20.04)

### Prerequirements

- See https://github.com/Phala-Network/phat-contract-examples for the phat contract development environment preparation.
- Some C toolchain also needed if it wasn't installed

  ```bash
  apt install clang clang-dev make
  ```

### Build commands

```bash
git clone https://github.com/Phala-Network/phat-quickjs.git
cd phat-quickjs
make
```

If no error happens, it should output normal compiled ink contract files in the path `target/ink`:

```
$ ls target/ink/
CACHEDIR.TAG  metadata.json  qjs.contract  qjs.wasm  release  wasm32-unknown-unknown
```

## Usage

Because this contract is imdeterministic, it can not be instantiated directly. Instead we need to upload the contract code to the cluster and then delegate call to the contract code with given code hash.

For example, suppose we have the contents in `metadata.json`:

```bash
$cat target/ink/metadata.json
{
  "source": {
    "hash": "0xc16b3166406fca22990acc417577ed2207415edb0665a59e58ef8e208713c279",
    "language": "ink! 3.4.0",
    "compiler": "rustc 1.68.0-nightly"
  },
  "contract": {
    "name": "qjs",
    "version": "0.1.0",
  },
  ...
}
```

We should first upload the contract to cluster, with js-sdk for example:

```javascript
const qjs = JSON.parse(fs.readFileSync("./target/qjs.contract"));
const code = qjs.source.wasm;
await assert.txAccepted(
  api.tx.phalaFatContracts.clusterUploadResource(
    clusterId,
    "IndeterministicInkCode",
    hex(code)
  ),
  alice
);
```

Then it can make delegate calls to the uploaded qjs code:

```rust
    #[derive(Debug, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Output {
        String(String),
        Bytes(Vec<u8>),
        Undefined,
    }

    #[ink(message)]
    pub fn eval_js(&self) {
        let delegate: ink_env::Hash = hex::decode("0xc16b3166406fca22990acc417577ed2207415edb0665a59e58ef8e208713c279").try_into().unwrap();
        let script = "console.log('Hello, World!')";
        let args: Vec<String> = vec![];
        use ink_env::call;

        let eval_js_selector = 0x49bfcd24_u32;
        let result = call::build_call::<pink::PinkEnvironment>()
            .call_type(call::DelegateCall::new().code_hash(delegate))
            .exec_input(
                call::ExecutionInput::new(call::Selector::new(eval_js_selector.to_be_bytes()))
                    .push_arg(script)
                    .push_arg(args),
            )
            .returns::<Result<Output, String>>()
            .fire();
        pink::info!("eval result: {result:?}");
    }
```

## Call other contracts in JavaScript
There are [two extra APIs](./npm_package/pink-env/src/index.ts) in this port of QuickJS that support calling other contracts in JavaScript.
For example:

```js
// Delegate calling
const delegateOutput = pink.invokeContractDelegate({
  codeHash: "0x0000000000000000000000000000000000000000000000000000000000000000",
  selector: 0xdeadbeef,
  input: "0x00"
});

// Instance calling
const contractOutput = pink.invokeContract({
  callee: "0x0000000000000000000000000000000000000000000000000000000000000000",
  input: "0x00",
  selector: 0xdeadbeef,
  gasLimit: 0n,
  value: 0n
});
```
