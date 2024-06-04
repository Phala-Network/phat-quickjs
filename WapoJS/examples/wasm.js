console.log = Wapo.inspect;

function assertEq(actual, expected, msg = "") {
  if (actual !== expected) {
    console.error(`Assertion failed: ${msg}, actual: ${actual}, expected: ${expected}`);
  }
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
  assertEq(global.value, 41);
  global.value = 42;
  assertEq(global.value, 42);
  assertEq(global.valueOf(), 42);
}

function test_validate() {
  console.log('# test_validate');
  const simpleWasm = WebAssembly.parseWat(wats.simple);
  const result = WebAssembly.validate(simpleWasm);
  assertEq(result, true);
  const module2 = new Uint8Array([
    0x00, 0x61, 0x73, 0x6d, // magic
    0x01, 0x00, 0x00, 0x00, // version
    0x01, 0x07, 0x01, 0x60, 0x00, 0x00, 0x0c,
  ]);
  const result2 = WebAssembly.validate(module2);
  assertEq(result2, false);
}

function test_load_module() {
  console.log('# test_load_module');
  const simpleWasm = WebAssembly.parseWat(wats.simple);
  const module = new WebAssembly.Module(simpleWasm);
  console.log('module.exports:', WebAssembly.Module.exports(module));
  console.log('module.imports:', WebAssembly.Module.imports(module));
}

async function test_compile() {
  console.log('# test_compile');
  const url = 'https://files.kvin.wang:8443/tests/simple.wasm';
  const module = await WebAssembly.compileStreaming(fetch(url));
  console.log('module.exports:', WebAssembly.Module.exports(module));
}

async function test_memory() {
  console.log('# test_memory');
  let memory = new WebAssembly.Memory({ initial: 1 });
  console.log('memory:', memory);
  let buffer = memory.buffer;
  console.log('buffer:', buffer.byteLength);
  assertEq(buffer.byteLength, 65536);
  memory.grow(1);
  console.log('buffer:', buffer.byteLength, buffer.detached);
  assertEq(buffer.byteLength, 0);
  assertEq(buffer.detached, true);
  console.log('memory.buffer:', memory.buffer.byteLength);
  assertEq(memory.buffer.byteLength, 131072);
}

function test_instance() {
  console.log('# test_instance');
  let module = new WebAssembly.Module(WebAssembly.parseWat(wats.simple));
  console.log('module.imports:', WebAssembly.Module.imports(module));
  let instance = new WebAssembly.Instance(module, {
    imports: {
      imported_func: function (arg) {
        console.log('imported_func:', arg);
      }
    }
  });
  console.log('instance.exports:', instance.exports);
  console.log('invoke ret:', instance.exports.exported_func());
  console.log('invoke ret:', instance.exports.exported_func(1));
  console.log('invoke ret:', instance.exports.exported_func(1, 2, 3));
}

function test_instance_global() {

  const global = new WebAssembly.Global({ value: "i32", mutable: true }, 0);

  WebAssembly.instantiateStreaming(WebAssembly.parseWat(wats.global), { js: { global } }).then(
    ({ instance }) => {
      assertEq(
        instance.exports.getGlobal(),
        0,
        "getting initial value from wasm",
      );
      global.value = 42;
      assertEq(
        instance.exports.getGlobal(),
        42,
        "getting JS-updated value from wasm",
      );
      instance.exports.incGlobal();
      assertEq(global.value, 43,
        "getting wasm-updated value from JS",
      );
      console.log('final global:', global.value, instance.exports.getGlobal());
    },
  );
}

async function test_fail() {
  console.log('# test_fail');
  const { instance } = await WebAssembly.instantiate(WebAssembly.parseWat(wats.fail));
  try {
    instance.exports.fail_me();
  } catch (e) {
    console.error('fail me error:', e);
  }
}

async function main() {
  test_global();
  test_validate();
  test_load_module();
  await test_compile();
  test_memory();
  test_instance();
  test_instance_global();
  await test_fail();
}

main().catch(console.error);