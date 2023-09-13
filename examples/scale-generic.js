const R = globalThis.Sidevm || globalThis.Pink;
const scl = R.SCALE;
const print = R.inspect;

function test(type, input, typeRegistry) {
    const encoded = scl.encode(input, type, typeRegistry);
    print(`encoded ${type}:`, encoded);
    const decoded = scl.decode(encoded, type, typeRegistry);
    print(`decoded ${type}:`, decoded);
}

const typeRegistry = `
Option<T> = <None|Some:T>;
JsError<T>      = <Ok:T|Err:str>;
OptionalString = Option<str>;
Info<A>   = {
    title: OptionalString,
    foo: JsError<()>,
    bar: {
        name: A
    },
    baz: {}
}
`
test('Info<u32>',
    {
        title: { Some: 'U32 name' },
        foo: { Ok: null },
        bar: { name: 123 }
    },
    typeRegistry
);
test('Info<str>',
    {
        title: { Some: 'str name' },
        foo: { Err: "Bug me" },
        bar: { name: "Tom" }
    },
    typeRegistry
);
