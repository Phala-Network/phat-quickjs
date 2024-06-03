function assertEqual(actual, expected) {
    if (actual !== expected) {
        throw new Error(`Expected ${expected}, but got ${actual}`);
    }
}

function test_global() {
    console.log('test_global');
    const global = new WebAssembly.Global({ value: "i32", mutable: true }, 41);
    console.log('init global:', global.value);
    assertEqual(global.value, 41);
    global.value = 42;
    assertEqual(global.value, 42);
    assertEqual(global.valueOf(), 42);
}

function main() {
    test_global();
}

main();