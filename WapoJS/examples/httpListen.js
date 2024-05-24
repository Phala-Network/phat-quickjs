console.log = Wapo.inspect;
console.log('Start to listen http requests...');

Wapo.httpListen(async req => {
    console.log('Incomming HTTP request:', req);

    var body = '';
    if (req.method === "POST") {
        body = await receiveBody(req.opaqueInputStream);
    }

    console.log('Received body of length:', body.length);
    Wapo.httpSendResponseHead(req.opaqueResponseTx, {
        status: 200,
        headers: {
            'Content-Type': 'text/plain',
            'X-Foo': 'Bar',
        }
    });
    const writer = Wapo.httpMakeWriter(req.opaqueOutputStream);
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
    console.log('Response sent, closing writer');
    Wapo.httpCloseWriter(writer);
});

async function receiveBody(streamHandle) {
    return new Promise((resolve, reject) => {
        const chunks = [];
        Wapo.httpReceiveBody(streamHandle, (cmd, data) => {
            switch (cmd) {
                case "data":
                    chunks.push(data);
                    console.log(`Received a chunk, length=${data.length}`);
                    break;
                case "error":
                    reject(data);
                    break;
                case "end":
                    resolve(Wapo.concatU8a(chunks));
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
        Wapo.httpWriteChunk(writer, data, (suc, err) => {
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
