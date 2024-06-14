const fs = require("fs");

function decodeAsset(asset) {
    // asset format:
    // data:application/wasm;base64,AGFzbQEAAAABnARDYA...
    return Buffer.from(asset.slice(asset.indexOf(",") + 1), "base64");
    // Or faster:
    // return Wapo.base64Decode(asset.slice(asset.indexOf(",") + 1), true);
}

fs.writeFileSync("/tfhe_bg.wasm", decodeAsset(require("node-tfhe/tfhe_bg.wasm")));