# phatjs-test

A command line tool for pink JavaScript testing.

## Installation

```sh
cargo install phatjs-test
```

## Usage

```
$ phatjs-test
Usage: phatjs-test [OPTIONS] <script> [script args...]

Options:
  -h, --help         Print this help message
  -v, --version      Print version info
  -l, --gas-limit    Set gas limit
  -j, --json         Output JSON
  --driver <driver>  Set driver. Available drivers: JsDelegate, JsDelegate2, AsyncJsRuntime. Default: JsDelegate
  -1                 Alias for --driver JsDelegate
  -2                 Alias for --driver JsDelegate2
  -3                 Alias for --driver AsyncJsRuntime
  -c <code>          Execute code directly
```

```
$ phatjs-test -2 dist/index.js 0x0000000000000000000000000000000000000000000000000000000000000003000000000000000000000000011c23b3aadaf3d4991f3abee262a34d18e9fdb5
2023-12-18T03:06:52.255486Z  INFO pink: [5CcS4dUYwcoU9QyHXtUhq2Jca5pMefxhgzoTenF2j2LSiYAj] evaluating js, code len: 33393    
2023-12-18T03:06:52.593962Z  INFO pink: [5CcS4dUYwcoU9QyHXtUhq2Jca5pMefxhgzoTenF2j2LSiYAj] JS: handle req: 0x0000000000000000000000000000000000000000000000000000000000000003000000000000000000000000011c23b3aadaf3d4991f3abee262a34d18e9fdb5    
2023-12-18T03:06:52.655431Z  INFO pink: [5CcS4dUYwcoU9QyHXtUhq2Jca5pMefxhgzoTenF2j2LSiYAj] JS: [3]: 0x011c23b3AadAf3D4991f3aBeE262A34d18e9fdb5    
2023-12-18T03:06:52.656154Z  INFO pink: [5CcS4dUYwcoU9QyHXtUhq2Jca5pMefxhgzoTenF2j2LSiYAj] JS: Malformed request received    
2023-12-18T03:06:52.681325Z  INFO phatjs_test: ExecResult: ContractResult { gas_consumed: Weight { ref_time: 26153706890132, proof_size: 711993 }, gas_required: Weight { ref_time: 26153706890132, proof_size: 1760479 }, storage_deposit: StorageDeposit::Charge(0), debug_message: [], result: Ok(ExecReturnValue { flags: (empty), data: [0, 0, 2, 9, 3, 48, 120, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 50, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 51, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48] }), events: None }

JS output: Ok(String("0x000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000000"))

==================== Gas usage ====================
Wall time         : 0.456s
Gas consumed      : 26153706890132 (26.154s)
Gas required      : 26153706890132
Max gas for query : 10000000000000
Gas required/query: 261.54%
Max gas for tx    : 500000000000
Gas required/tx   : 5230.74%
Time scaled       : 57.36x
===================================================
```

```
$ phatjs-test -j -3 dist/index.js 0x0000000000000000000000000000000000000000000000000000000000000003000000000000000000000000011c23b3aadaf3d4991f3abee262a34d18e9fdb5 2>/dev/null

{
  "contractExecResult": "0x0765f22a6105162501000765f22a61054224410001000000000000000000000000000000000000000000001d030000020903307830303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303032303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030333030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303000",
  "gasConsumed": 23105040997,
  "gasRequired": 23105040997,
  "queryGasLimit": 10000000000000,
  "txGasLimit": 500000000000,
  "wallTimeUs": 315050,
  "jsOutput": {
    "Ok": {
      "string": "0x000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000000"
    }
  }
}
```
