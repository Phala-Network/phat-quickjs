console.log = Sidevm.inspect;
console.log('Config: ', globalThis.config);
console.log('Start to listen http requests...');

Sidevm.httpListen(async req => {
    console.log('Incomming HTTP request:', req);

    var body = '';
    if (req.method === "POST") {
        body = await receiveBody(req.opaqueInputStream);
    }

    console.log('Received body of length:', body.length);
    Sidevm.httpSendResponseHead(req.opaqueResponseTx, {
        status: 200,
        headers: {
            'Content-Type': 'text/plain',
            'X-Foo': 'Bar',
        }
    });
    const writer = Sidevm.httpMakeWriter(req.opaqueOutputStream);
    const url = new URL(req.url);
    console.log('path: ', url.pathname);
    if (url.pathname == '/_/getConfig') {
        await writeString(writer, JSON.stringify(globalThis.config));
    } else {
        await writeString(writer, `You have sent me the following info\n`);
        await writeString(writer, `method: ${req.method}\n`);
        await sleep(1000);
        await writeString(writer, `url: ${req.url}\n`);
        await sleep(1000);
        await writeString(writer, `headers: \n`);
        for (var p of req.headers) {
            await writeString(writer, `    ${p[0]}: ${p[1]}\n`);
            await sleep(500);
        }
        await writeString(writer, `actual body length: ${body.length}\n`);
        await writeString(writer, `My config: ${globalThis.config}\n`);
    }
    console.log('Response sent, closing writer');
    Sidevm.httpCloseWriter(writer);
});

async function receiveBody(streamHandle) {
    return new Promise((resolve, reject) => {
        const chunks = [];
        Sidevm.httpReceiveBody(streamHandle, (cmd, data) => {
            switch (cmd) {
                case "data":
                    chunks.push(data);
                    console.log(`Received a chunk, length=${data.length}`);
                    break;
                case "error":
                    reject(data);
                    break;
                case "end":
                    resolve(Sidevm.concatU8a(chunks));
                    break;
                default:
                    console.log("unknown cmd:", cmd);
                    break;
            }
        });
    });
}

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
