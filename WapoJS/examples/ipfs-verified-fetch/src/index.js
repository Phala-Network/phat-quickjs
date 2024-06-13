import { createVerifiedFetch } from '@helia/verified-fetch'

function addListener(callback) {
    Wapo.httpListen(req => {
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
                    headers: Array.from(response.headers.entries()),
                });
                if (response._opaqueBodyStream) {
                    // offload to host for better performance
                    Wapo.streamBridge({
                        input: response._opaqueBodyStream,
                        output: req.opaqueOutputStream,
                    });
                } else {
                    const writer = toWritableStream(req.opaqueOutputStream);
                    response.body.pipeTo(writer);
                }
            }
        }
        callback(event);
    });
}

(async function() {
    const verifiedFetch = await createVerifiedFetch({
        gateways: ['https://trustless-gateway.link', 'https://cloudflare-ipfs.com'],
    })
    const resp = await verifiedFetch('ipfs://baguqeeradnk3742vd3jxhurh22rgmlpcbzxvsy3vc5bzakwiktdeplwer6pa');
    console.log(await resp.text());
    console.log(Wapo);
    console.log(Wapo.httpListen);
    
    addListener(async event => {
        const request = event.request;
        const url = new URL(request.url);
        switch (url.pathname) {
        case "/":
            event.respondWith("Hello, World!");
            break;
        default:
            event.respondWith("404");
            break;
	}
    });
}());

//main().catch(console.error).finally(() => process.exit());
