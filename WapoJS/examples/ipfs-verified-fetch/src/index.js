import { createVerifiedFetch } from '@helia/verified-fetch'

console.log = Wapo.inspect

function addEventListener(callback) {
    Wapo.httpListen(req => {
        console.log('incoming httpListen request')
        const request = {
            url: req.url,
            method: req.method,
            headers: req.headers,
            body: toReadableStream(req.opaqueInputStream),
        };
        const event = {
            type: "fetch",
            request,
            async respondWith(response) {
                response = await response;
                Wapo.httpSendResponseHead(req.opaqueResponseTx, {
                    status: response.status,
                    headers: response.headers,
                });
                const writer = toWritableStream(req.opaqueOutputStream);
                response.body.pipeTo(writer);
            }
        }
        callback(event);
    });
}


function toReadableStream(body) {
    return new ReadableStream({
        start(controller) {
            Wapo.streamOpenRead(body, (cmd, data) => {
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

function toWritableStream(streamId) {
    const writer = Wapo.streamOpenWrite(streamId);
    return new WritableStream({
        write(chunk) {
            return new Promise((resolve, reject) => {
                Wapo.streamWriteChunk(writer, chunk, (suc, err) => {
                    if (suc) {
                        resolve();
                    } else {
                        reject(err);
                    }
                });
            });
        },
        close() {
            Wapo.streamClose(writer);
        }
    });
}

(async function() {
    const verifiedFetch = await createVerifiedFetch({
        gateways: ['https://trustless-gateway.link', 'https://cloudflare-ipfs.com'],
    })
    const resp = await verifiedFetch('ipfs://baguqeeradnk3742vd3jxhurh22rgmlpcbzxvsy3vc5bzakwiktdeplwer6pa');
    console.log(await resp.text());
    console.log('keys:', Object.keys(Wapo));
    console.log(Wapo.httpListen);
    
    addEventListener(async event => {
        console.log('incoming request')
        const request = event.request;
        const url = new URL(request.url);
        switch (url.pathname) {
        case "/":
            event.respondWith(new Response("Hello, World!"));
            break;
        default:
            event.respondWith(new Response("404"));
            break;
	}
    });
}());

//main().catch(console.error).finally(() => process.exit());
