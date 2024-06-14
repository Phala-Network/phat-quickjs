const fs = require("fs");

function toBytes(s) {
    return new TextEncoder().encode(s);
}

fs.writeFileSync("/tfhe_bg.wasm", toBytes(require("node-tfhe/tfhe_bg.wasm")));
