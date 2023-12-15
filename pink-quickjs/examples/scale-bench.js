(function() {
    const delegate = pink.SCALE.encodeAll ? "JsDelegate2" : "JsDelegate";

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
    const iterations = 50;
    const t0 = Date.now();
    for (var i = 0; i < iterations; i++) {
        pink.SCALE.parseTypes(types);
    }
    console.log(`${delegate}: parse ${iterations} iterations in ${Date.now() - t0}ms, ${iterations / (Date.now() - t0) * 1000} ops/sec`);
    const parsedTypes = pink.SCALE.parseTypes(types);
    const data = [];
    for (var i = 0; i < 20; i++) {
        data.push({ foo: 1, bar: 'baz' }); 
    }
    const t1 = Date.now();
    const coder = pink.SCALE.codec(6, parsedTypes);
    for (var i = 0; i < iterations; i++) {
        const encoded = coder.encode(data);
        const decoded = coder.decode(encoded);
        if (decoded.length != data.length) {
            throw new Error(`Decoded length mismatch: ${decoded.length} != ${data.length}`);
        }
    }
    console.log(`${delegate}: enc/dec ${iterations} iterations in ${Date.now() - t1}ms, ${iterations / (Date.now() - t1) * 1000} ops/sec`);
}());

