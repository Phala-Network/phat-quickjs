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

const wats = {
    global: `
    (module
        (global $g (import "js" "global") (mut i32))
        (func (export "getGlobal") (result i32)
        (global.get $g)
        )
        (func (export "incGlobal")
        (global.set $g (i32.add (global.get $g) (i32.const 1)))
        )
    )
    `,
    fail: `
    (module
        (func (export "fail_me") (result i32)
        i32.const 1
        i32.const 0
        i32.div_s
        )
    )
    `,
    memory: `
    (module
        (memory (import "js" "mem") 1)
        (func (export "accumulate") (param $ptr i32) (param $len i32) (result i32)
          (local $end i32)
          (local $sum i32)
          (local.set $end
            (i32.add
              (local.get $ptr)
              (i32.mul
                (local.get $len)
                (i32.const 4))))
          (block $break
            (loop $top
              (br_if $break
                (i32.eq
                  (local.get $ptr)
                  (local.get $end)))
              (local.set $sum
                (i32.add
                  (local.get $sum)
                  (i32.load
                    (local.get $ptr))))
              (local.set $ptr
                (i32.add
                  (local.get $ptr)
                  (i32.const 4)))
              (br $top)
            )
          )
          (local.get $sum)
        )
      )
    `,
    simple: `
    (module
        (func $i (import "imports" "imported_func") (param i32))
        (func (export "exported_func")
          i32.const 42
          call $i
        )
      )
    `,
    table: `
    (module
        (func $thirteen (result i32) (i32.const 13))
        (func $fourtytwo (result i32) (i32.const 42))
        (table (export "tbl") anyfunc (elem $thirteen $fourtytwo))
      )
    `,
    table2: `
    (module
        (import "js" "tbl" (table 2 anyfunc))
        (func $f42 (result i32) i32.const 42)
        (func $f83 (result i32) i32.const 83)
        (elem (i32.const 0) $f42 $f83)
      )
    `,
}

function test_global() {
    console.log('# test_global');
    const global = new WebAssembly.Global({ value: "i32", mutable: true }, 41);
    console.log('init global:', global.value);
    assertEqual(global.value, 41);
    global.value = 42;
    assertEqual(global.value, 42);
    assertEqual(global.valueOf(), 42);
}

function test_validate() {
    console.log('# test_validate');
    const simpleWasm = WebAssembly.parseWat(wats.simple);
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
    console.log('# test_load_module');
    const simpleWasm = WebAssembly.parseWat(wats.simple);
    const module = new WebAssembly.Module(simpleWasm);
    console.log('module.exports:', WebAssembly.Module.exports(module));
    console.log('module.imports:', WebAssembly.Module.imports(module));
}

function main() {
    test_global();
    test_validate();
    test_load_module();
}

main();