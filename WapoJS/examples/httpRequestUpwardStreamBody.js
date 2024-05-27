console.log = Wapo.inspect;
const url = "https://httpbin.kvin.wang:8443/anything";
console.log('Posting data to url:', url);

(async () => {
    const request = {
        url,
        method: "POST",
        headers: {
            "X-Foo": "Bar",
            "Content-Type": "text/plain",
        },
        streamBody: true
    };
    const req = Wapo.httpRequest(request,
        (cmd, data) => {
            console.log(`=================[${cmd}]===================`);
            switch (cmd) {
                case "head":
                    console.log("head:", data);
                    Wapo.httpReceiveBody(data.opaqueBodyStream, (cmd, data) => {
                        switch (cmd) {
                            case "data":
                                console.log(`data.length=${data.length}`);
                                console.log('-------------------------------------------');
                                console.log(new TextDecoder().decode(data));
                                break;
                            case "end":
                                break;
                            case "error":
                                console.log("error:", data);
                                break;
                        }
                    });
                    break;
                case "error":
                    console.log("error:", data);
                    break;
                default:
                    console.log("unknown cmd:", cmd);
                    console.log("data:", data);
                    break;
            }
        }
    );
    await writeBody(req);
})();

async function sleep(ms) {
    return new Promise((resolve) => {
        setTimeout(resolve, ms);
    });
}

async function writeBody(req) {
    const writer = Wapo.httpMakeWriter(req.opaqueBodyStream);
    for (let i = 0; i < 10; i++) {
        await writeString(writer, `HelloWorld\n`);
        await sleep(1000);
    }
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