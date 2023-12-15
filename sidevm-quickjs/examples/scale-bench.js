(function() {
    const Runtime = globalThis.pink ?? globalThis.Sidevm;
    const scl = Runtime.SCALE;
    const envName = globalThis.pink ? scl.encodeAll ? "phat-qjs v2" : "phat-qjs v1" : "Sidevm qjs";

    var types = `
#u8
#str
(0,1)
<Ok:0,Err:1>
[0]
[1]
[7]
{foo:0,bar:1}
`;
    for (var i = 0; i < 20; i++) {
        types += `{foo:0,bar:1}\n`;
    }
    const iterations = 100;
    const t0 = Date.now();
    for (var i = 0; i < iterations; i++) {
        scl.parseTypes(types);
    }
    console.log(`${envName}: parse ${iterations} iterations in ${Date.now() - t0}ms, ${iterations / (Date.now() - t0) * 1000} ops/sec`);
    const parsedTypes = scl.parseTypes(types);
    const data = [];
    for (var i = 0; i < 20; i++) {
        data.push({ foo: 1, bar: 'baz' }); 
    }
    const t1 = Date.now();
    const coder = scl.codec(6, parsedTypes);
    for (var i = 0; i < iterations; i++) {
        const encoded = coder.encode(data);
        const decoded = coder.decode(encoded);
        if (decoded.length != data.length) {
            throw new Error(`Decoded length mismatch: ${decoded.length} != ${data.length}`);
        }
    }
    console.log(`${envName}: enc/dec ${iterations} iterations in ${Date.now() - t1}ms, ${iterations / (Date.now() - t1) * 1000} ops/sec`);
}());

