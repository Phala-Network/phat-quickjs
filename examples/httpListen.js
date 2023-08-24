console.log = Sidevm.inspect;
console.log('Start to listen http requests...');
Sidevm.httpListen((req) => {
    console.log('Incomming HTTP request:', req);
    Sidevm.httpSendResponse(req.opaqueResponseTx, {
        status: 200,
        headers: {
            'Content-Type': 'text/plain',
            'X-Foo': 'Bar',
        }
    })
    let chunks = [
        "Hello, \n",
        "World!\n",
        "This is a test.\n",
    ];
    const writer = Sidevm.httpMakeWriter(req.opaqueOutputStream);
    const marker2 = Sidevm.marker("marker2");
    const p = {
        writeAll(data) {
            console.log('marker2', marker2);
            const chunk = new TextEncoder().encode(data[0]);
            Sidevm.httpWriteChunk(writer, chunk, (suc, err) => {
                const rest = data.slice(1);
                if (rest.length > 0) {
                    setTimeout(p.writeAll, 1000, rest);
                } else {
                    p.writeAll = () => { };
                }
            });
        }
    };
    p.writeAll(chunks);
});
