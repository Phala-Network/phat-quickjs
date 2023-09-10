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
    testPerson();

})();