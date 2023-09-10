console.log = Sidevm.inspect;
const scl = Sidevm.SCALE;

(() => {
    const anonTypes = `
#u8
#str
(0,1)
<Ok:0,Err:1>
[0]
[1]
[0;2]
{foo:0,bar:1}
`

    let registry = scl.parseTypes(anonTypes)
    console.log("registry:", registry);

    function testEnum() {
        console.log("===========");
        console.log("testEnum");
        let orig = { Ok: 9 };
        console.log("orig:", orig);
        const encoded = scl.encode(orig, 3, registry);
        console.log("encoded:", encoded);
        const decoded = scl.decode(encoded, 3, registry);
        console.log("decoded:", decoded);
    }

    function testU8a() {
        console.log("===========");
        console.log("testU8a");
        var orig = new Uint8Array([1, 2, 3]);
        console.log("orig:", orig);
        var encoded = scl.encode(orig, 4, registry);
        console.log("encoded:", encoded);
        var decoded = scl.decode(encoded, 4, registry);
        console.log("decoded:", decoded);

        orig = "0x020304";
        console.log("orig:", orig);
        encoded = scl.encode(orig, 4, registry);
        console.log("encoded:", encoded);
        decoded = scl.decode(encoded, 4, registry);
        console.log("decoded:", decoded);
    }

    function testArray() {
        console.log("===========");
        console.log("testArray");
        let orig = [1, 2];
        console.log("orig:", orig);
        const encoded = scl.encode(orig, 6, registry);
        console.log("encoded:", encoded);
        const decoded = scl.decode(encoded, 6, registry);
        console.log("decoded:", decoded);
    }

    function testTuple() {
        console.log("===========");
        console.log("testTuple");
        let orig = [1, "hello"];
        console.log("orig:", orig);
        const encoded = scl.encode(orig, 2, registry);
        console.log("encoded:", encoded);
        const decoded = scl.decode(encoded, 2, registry);
        console.log("decoded:", decoded);
    }

    function testStrArray() {
        console.log("===========");
        console.log("testStrArray");
        let orig = ["foo", "bar", "baz"];
        console.log("orig:", orig);
        const encoded = scl.encode(orig, 5, registry);
        console.log("encoded:", encoded);
        const decoded = scl.decode(encoded, 5, registry);
        console.log("decoded:", decoded);
    }

    testArray();
    testTuple();
    testStrArray();
    testU8a();
    testEnum();


})();

(() => {
    const anonTypes = `
    Age = u32
    Person = {
        name:str,
        age:Age,
    }
`

    let registry = scl.parseTypes(anonTypes)
    console.log("registry:", registry);

    function testPerson() {
        console.log("===========");
        console.log("testPerson");
        let orig = { name: "Tom", age: 9 };
        console.log("orig:", orig);
        const encoded = scl.encode(orig, "Person", registry);
        console.log("encoded:", encoded);
        const decoded = scl.decode(encoded, "Person", registry);
        console.log("decoded:", decoded);
    }
    function testImediateDef() {
        console.log("===========");
        console.log("testImediateDef");
        {
            let orig = "Hello world!"
            console.log("orig:", orig);
            const encoded = scl.encode(orig, "str");
            console.log("encoded:", encoded);
            const decoded = scl.decode(encoded, "str");
            console.log("decoded:", decoded);
        }
        console.log("------------");
        {
            let orig = { name: "Tom", age: 9n };
            console.log("orig:", orig);
            const encoded = scl.encode(orig, '{name:str,age:u32}');
            console.log("encoded:", encoded);
            const decoded = scl.decode(encoded, '{name:str,age:u32}');
            console.log("decoded:", decoded);
        }
        console.log("------------");
        // Recursive
        {
            let orig = { name: "Tom", age: 9, cards: { foo: "fooz", bar: 42 } };
            console.log("orig:", orig);
            const typedef = '{name:str,age:u32,cards:{foo:str,bar:u8}}';
            const encoded = scl.encode(orig, typedef);
            console.log("encoded:", encoded);
            const decoded = scl.decode(encoded, typedef);
            console.log("decoded:", decoded);
        }
    }
    function testCodecObj() {
        console.log("===========");
        {
            const registry = scl.parseTypes(`Person={name:str,age:u8}`)
            const coder = scl.codec(['Person'], registry);
            console.log("coder:", coder);
            let orig = { name: "Tom", age: 9 };
            console.log("orig:", orig);
            const encoded = coder.encode([orig]);
            console.log("encoded:", encoded);
            const decoded = coder.decode(encoded);
            console.log("decoded:", decoded);
            console.log("encoded again:", coder.encode(decoded));
        }
    }
    testPerson();
    testImediateDef();
    testCodecObj();

})();