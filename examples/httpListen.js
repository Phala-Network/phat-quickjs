console.log = Sidevm.inspect;
console.log('Start to listen http requests...');

Sidevm.httpListen(async (req) => {
    console.log('Incomming HTTP request:', req);
    Sidevm.httpSendResponse(req.opaqueResponseTx, {
        status: 200,
        headers: {
            'Content-Type': 'text/plain',
            'X-Foo': 'Bar',
        }
    })
    const writer = Sidevm.httpMakeWriter(req.opaqueOutputStream);
    await writeString(writer, `method: ${req.method}\n`);
    await sleep(1000);
    await writeString(writer, `path: ${req.path}\n`);
    await sleep(1000);
    await writeString(writer, `query: ${req.query}\n`);
    await sleep(1000);
    await writeString(writer, `headers: \n`);
    for (var p of Object.entries(req.headers)) {
        await writeString(writer, `    ${p[0]}: ${p[1]}\n`);
        await sleep(500);
    }
});



async function writeString(writer, s) {
    const data = new TextEncoder().encode(s);
    return new Promise((resolve, reject) => {
        Sidevm.httpWriteChunk(writer, data, (suc, err) => {
            if (suc) {
                resolve();
            } else {
                reject(err);
            }
        });
    });
}

async function sleep(ms) {
    return new Promise((resolve) => {
        setTimeout(resolve, ms);
    });
}
