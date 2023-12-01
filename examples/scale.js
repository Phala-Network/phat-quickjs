const Runtime = globalThis.Pink || globalThis.Sidevm;
const repr = Runtime.repr;
const scl = Runtime.SCALE;
console.log = Runtime.inspect;

// Helper functions for colorizing console output
function green(str) {
    return `\x1b[32m${str}\x1b[0m`;
}

function red(str) {
    return `\x1b[31m${str}\x1b[0m`;
}

const typeRegistry0 = `
#u8
#str
(0,1)
<Ok:0,Err:1>
[0]
[1]
[0;2]
{foo:0,bar:1}
    `;
const registry0Tests = [
    { input: { Ok: 9 }, type: 3, title: 'Encoding a enum' },
    { input: new Uint8Array([1, 2, 3]), type: 4, title: 'Encoding a Uint8Array' },
    { input: [1, 2, 3], type: 4, title: 'Encoding a seq u8', u8a: true },
    { input: [1, 2], type: 6, title: 'Encoding an Array', u8a: true },
    { input: [1, "hello"], type: 2, title: 'Encoding a Tuple' },
    { input: ["foo", "bar", "baz"], type: 5, title: 'Encoding a String Array' },
];
const typeRegistry1 = `
    Age = u32
    Person = {
        name:str,
        age:Age,
    }
`
const registry1Tests = [
    { input: "Hello world!", type: "str", title: 'Encoding String' },
    { input: { name: "Tom", age: 9 }, type: "Person", title: 'Encoding Person Object' },
    { input: { name: "Tom", age: 9n }, type: '{name:str,age:u32}', title: 'Encoding Immediate Definition' },
    { input: { name: "Tom", age: 9, cards: { foo: "fooz", bar: 42 } }, type: '{name:str,age:u32,cards:{foo:str,bar:u8}}', title: 'Encoding Recursive Definition' },
    { input: { name: "Tom", age: 1000000000000000000000000000n }, type: '{name:str,age:u128}', title: 'Encoding with big number' },
    { input: { name: "Tom", age: 10n }, type: '{name:str,age:@u128}', title: 'Encoding compact u128' },
    { input: [1, 2, 3, 4], type: '[u32]', title: 'Encoding [u32]' },
    { input: [1, 'foo', { name: "Tom", age: 10n }], type: ['u8', 'str', 'Person'], title: 'Encoding multiple types' }
];

function assertEqual(expected, actual, title) {
    const reprExpected = repr(expected);
    const reprActual = repr(actual);
    if (reprExpected == reprActual) {
        console.log(`${title} [${green('OK')}]`);
    } else {
        console.error(`${title} [${red('FAIL')}]`);
        console.error(`Expected: ${reprExpected}`);
        console.error(`Actual  :   ${reprActual}`);
    }
}

function runTests(registry, cases) {
    const parsedRegistry = scl.parseTypes(registry);
    cases.forEach(({ input, type, title, u8a }, index) => {
        console.log();
        console.log(`=================[ ${index + 1}: ${title} ]===================`);
        console.log(`Input: ${repr(input)}`);
        const encode = Array.isArray(type) ? scl.encodeAll : scl.encode;
        const decode = Array.isArray(type) ? scl.decodeAll : scl.decode;
        const encode0 = encode(input, type, parsedRegistry);
        const encode1 = encode(input, type, registry);
        assertEqual(encode0, encode1, 'Encoding results equal');
        const decode0 = decode(encode0, type, parsedRegistry);
        const decode1 = decode(encode1, type, registry);
        assertEqual(decode0, decode1, 'Decoding results equal');
        if (!u8a) {
            assertEqual(decode0, input, 'Decoding equals input');
        }

        const coder = scl.codec(type, registry);
        const encode2 = coder.encode(input);
        assertEqual(encode0, encode2, 'Encoding with codec results equal');
        const decode2 = coder.decode(encode2);
        assertEqual(decode0, decode2, 'Decoding with codec results equal');
    });
}

// Run the tests
runTests(typeRegistry0, registry0Tests);
runTests(typeRegistry1, registry1Tests);
