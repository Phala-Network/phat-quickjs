console.log("Listening for fetch");
addEventListener("fetch", async (event) => {
    const request = event.request;
    console.log("Incoming fetch event");
    if (request.method = "POST") {
        for await (const chunk of request.body) {
            console.log("Received chunk of length:", chunk.length);
        }
    }
    const headers = request.headers;
    event.respondWith({
        status: 200,
        headers: {
            "Content-Type": "text/plain",
            "X-Foo": "Bar",
        },
        body: new ReadableStream({
            async start(controller) {
                for (var p of headers) {
                    controller.enqueue(new TextEncoder().encode(`  ${p[0]}: ${p[1]}\n`));
                    await sleep(1000);
                }
                controller.close();
            }
        })
    });
});

function toReadableStream(body) {
    return new ReadableStream({
        start(controller) {
            Sidevm.httpReceiveBody(body, (cmd, data) => {
                switch (cmd) {
                    case "data":
                        controller.enqueue(data);
                        break;
                    case "end":
                        controller.close();
                        break;
                    case "error":
                        controller.error(data);
                        break;
                    default:
                        console.log("unknown cmd:", cmd);
                        break;
                }
            });
        }
    });
}

function toWritableStream(writer) {
    return new WritableStream({
        write(chunk) {
            return new Promise((resolve, reject) => {
                Sidevm.httpWriteChunk(writer, chunk, (suc, err) => {
                    if (suc) {
                        resolve();
                    } else {
                        reject(err);
                    }
                });
            });
        },
        close() {
            Sidevm.httpCloseWriter(writer);
        }
    });
}

function addEventListener(type, callback) {
    switch (type) {
        case "fetch":
            {
                Sidevm.httpListen((req) => {
                    const request = {
                        url: 'https://localhost' + req.path, // TODO: add query and origin
                        method: req.method,
                        headers: req.headers,
                        body: toReadableStream(req.opaqueInputStream),
                    };
                    const event = {
                        type: "fetch",
                        request,
                        respondWith(response) {
                            Sidevm.httpSendResponse(req.opaqueResponseTx, {
                                status: response.status,
                                headers: response.headers,
                            });
                            const writer = toWritableStream(Sidevm.httpMakeWriter(req.opaqueOutputStream));
                            response.body.pipeTo(writer);
                        }
                    }
                    callback(event);
                });
            }
            break;
        default:
            throw new Error(`unknown event type: ${type}`);
    }
}

async function sleep(ms) {
    return new Promise((resolve) => {
        setTimeout(resolve, ms);
    });
}
