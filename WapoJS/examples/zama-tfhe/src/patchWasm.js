const fs = require("fs");

fs.writeFileSync("/tfhe_bg.wasm", require("node-tfhe/tfhe_bg.wasm"));
