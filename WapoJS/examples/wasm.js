console.log = Wapo.inspect;

function assertEqual(actual, expected) {
    if (actual !== expected) {
        throw new Error(`Expected ${expected}, but got ${actual}`);
    }
}

function hexDecode(hex) {
    // Remove all non-hexadecimal characters (like spaces and newlines)
    hex = hex.replace(/[^0-9a-fA-F]/g, '');

    return new Uint8Array(hex.match(/.{1,2}/g).map(byte => parseInt(byte, 16)));
}

const simpleWasm = hexDecode(`
    00 61 73 6d 01 00 00 00  01 07 01 60 02 7f 7f 01
    7f 03 02 01 00 05 03 01  00 10 06 19 03 7f 01 41
    80 80 c0 00 0b 7f 00 41  80 80 c0 00 0b 7f 00 41
    80 80 c0 00 0b 07 2b 04  06 6d 65 6d 6f 72 79 02
    00 03 61 64 64 00 00 0a  5f 5f 64 61 74 61 5f 65
    6e 64 03 01 0b 5f 5f 68  65 61 70 5f 62 61 73 65
    03 02 0a 09 01 07 00 20  01 20 00 6a 0b 00 2f 04
    6e 61 6d 65 00 0c 0b 73  69 6d 70 6c 65 2e 77 61
    73 6d 01 06 01 00 03 61  64 64 07 12 01 00 0f 5f
    5f 73 74 61 63 6b 5f 70  6f 69 6e 74 65 72 00 3d
    09 70 72 6f 64 75 63 65  72 73 01 0c 70 72 6f 63
    65 73 73 65 64 2d 62 79  01 05 72 75 73 74 63 1d
    31 2e 37 38 2e 30 20 28  39 62 30 30 39 35 36 65
    35 20 32 30 32 34 2d 30  34 2d 32 39 29 00 2c 0f
    74 61 72 67 65 74 5f 66  65 61 74 75 72 65 73 02
    2b 0f 6d 75 74 61 62 6c  65 2d 67 6c 6f 62 61 6c
    73 2b 08 73 69 67 6e 2d  65 78 74               
`);

function test_global() {
    console.log('test_global');
    const global = new WebAssembly.Global({ value: "i32", mutable: true }, 41);
    console.log('init global:', global.value);
    assertEqual(global.value, 41);
    global.value = 42;
    assertEqual(global.value, 42);
    assertEqual(global.valueOf(), 42);
}

function test_validate() {
    console.log('test_validate');
    const result = WebAssembly.validate(simpleWasm);
    assertEqual(result, true);
    const module2 = new Uint8Array([
        0x00, 0x61, 0x73, 0x6d, // magic
        0x01, 0x00, 0x00, 0x00, // version
        0x01, 0x07, 0x01, 0x60, 0x00, 0x00, 0x0c,
    ]);
    const result2 = WebAssembly.validate(module2);
    assertEqual(result2, false);
}

function test_load_module() {
    console.log('test_load_module');
    const module = new WebAssembly.Module(simpleWasm);
    console.log('module:', module);
    console.log('module.exports:', WebAssembly.Module.exports(module));
}

function main() {
    test_global();
    test_validate();
    test_load_module();
}

main();