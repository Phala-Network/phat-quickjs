(async function () {
    // The private key is kept in the host side. Js code can only access the public key.
    const ecdhKey = await crypto.subtle.generateKey(
        {
            name: "ECDH",
            namedCurve: "P-384",
        },
        false,
        ["deriveKey"]
    );

    const signedPublicKey = await signPublicKey(ecdhKey.publicKey);
    const config = {
        apiKey: "",
        apiUrl: "https://api.openai.com/v1/chat/completions",
    };

    addEventListener("fetch", async event => {
        const request = event.request;
        const url = new URL(request.url);
        switch (url.pathname) {
            case "/":
                event.respondWith(new Response(indexPage(), {
                    headers: {
                        "Content-Type": "text/html",
                    },
                }));
                break;
            case "/publicKey":
                event.respondWith(new Response(signedPublicKey, {
                    headers: {
                        "Content-Type": "application/json",
                    },
                }));
                break;
            case "/config": {
                const body = new TextDecoder().decode(await readAll(request.body));
                const { publicKey, ciphertext, iv } = JSON.parse(body);
                // Convert hexed publicKey to ArrayBuffer
                const clientPubkey = fromHexString(publicKey);

                // Import client's public key
                const clientPublicKey = await crypto.subtle.importKey(
                    "raw",
                    clientPubkey,
                    {
                        name: "ECDH",
                        namedCurve: "P-384",
                    },
                    false,
                    []
                );
                // Derive shared secret key
                const derivedKey = await crypto.subtle.deriveKey(
                    {
                        name: "ECDH",
                        public: clientPublicKey,
                    },
                    ecdhKey.privateKey,
                    {
                        name: "AES-GCM",
                        length: 256,
                    },
                    false,
                    ["decrypt"]
                );

                // Decrypt ciphertext using derived key and provided IV
                const decipherOptions = {
                    name: "AES-GCM",
                    iv: fromHexString(iv),
                };
                const decryptedData = await crypto.subtle.decrypt(
                    decipherOptions,
                    derivedKey,
                    fromHexString(ciphertext)
                );
                // Convert decrypted data to string and store in config
                const configUpdate = JSON.parse(new TextDecoder().decode(decryptedData));
                if (configUpdate.apiKey) {
                    config.apiKey = configUpdate.apiKey;
                    console.log("API Key updated");
                }
                if (configUpdate.apiUrl) {
                    config.apiUrl = configUpdate.apiUrl;
                    console.log("API URL set to:", config.apiUrl);
                }
                event.respondWith(new Response('{"status": "ok"}'), {
                    headers: {
                        "Content-Type": "application/json",
                    },
                });
                break;
            }
            case "/v1/chat/completions": {
                (async function () {
                    const headers = {
                        "Authorization": `Bearer ${config.apiKey}`,
                    };
                    const body = await readAll(request.body);
                    const response = await fetch(config.apiUrl, {
                        method: request.method,
                        headers,
                        body,
                    });
                    event.respondWith(response);
                }());
                break;
            }
            default:
                event.respondWith(new Response("Not found", {
                    status: 404,
                }));
                break;
        }
    });
    addEventListener("query", async event => {
        switch (event.request.path) {
            case "/":
                event.respondWith("Hello, World!");
                break;
            case "/publicKey":
                event.respondWith(signedPublicKey);
                break;
            default:
                event.respondWith("404");
                break;
        }
    });
}());

async function readAll(stream) {
    const reader = stream.getReader();
    const chunks = [];
    while (true) {
        const { done, value } = await reader.read();
        if (done) {
            break;
        }
        chunks.push(value);
    }
    return Wapo.concatU8a(chunks);
}

async function signPublicKey(publicKey) {
    const ecdhPublicKey = toHexString(await crypto.subtle.exportKey("raw", publicKey));
    const contentToSign = Wapo.hash("sha256", "ecdhPublicKey:" + ecdhPublicKey);
    const signature = Wapo.workerSign(contentToSign);
    const workerPublicKey = Wapo.workerPublicKey();
    const sgxQuote = Wapo.sgxQuote(contentToSign);

    return JSON.stringify({
        ecdhPublicKey,
        signature: toHexString(signature),
        workerPublicKey: toHexString(workerPublicKey),
        sgxQuote: toHexString(sgxQuote),
    });
}

function toHexString(buffer) {
    return Array.prototype.map.call(new Uint8Array(buffer), x => ('00' + x.toString(16)).slice(-2)).join('');
}

function fromHexString(hexString) {
    return new Uint8Array(hexString.match(/.{1,2}/g).map(byte => parseInt(byte, 16)));
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

function addEventListener(type, callback) {
    switch (type) {
        case "fetch":
            {
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
                break;
            }
        case "query":
            {
                Wapo.queryListen(request => {
                    const event = {
                        type: "query",
                        request: {
                            origin: request.origin,
                            path: request.path,
                            payload: request.payload,
                        },
                        async respondWith(data) {
                            Wapo.queryReply(request.replyTx, data);
                        }
                    }
                    callback(event);
                });
                break;
            }
        default:
            throw new Error(`unknown event type: ${type}`);
    }
}

function indexPage() {
    return `
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Console</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                }
                .console {
                    width: 80%;
                    margin: 20px auto;
                    padding: 10px;
                    background-color: #f0f0f0;
                    border: 1px solid #ccc;
                    border-radius: 5px;
                }
                .console textarea {
                    width: 100%;
                    height: 150px;
                }
                .console button {
                    display: block;
                    margin: 10px 0;
                    padding: 10px;
                    background-color: #007BFF;
                    color: white;
                    border: none;
                    border-radius: 5px;
                    cursor: pointer;
                }
                .console button:hover {
                    background-color: #0056b3;
                }
                .output {
                    background-color: #e0e0e0;
                }
            </style>
        </head>
        <body>
            <div class="console">
                <h2>Debug Console (Don't use in production) </h2>
                <textarea id="config-input" placeholder="Enter JSON Config">
                {
                    "apiKey": "",
                    "apiUrl": "https://api.red-pill.ai/v1/chat/completions"
                }
                </textarea>
                <button onclick="updateConfig()">Update Config</button>
                <textarea id="message-input" placeholder="Enter OpenAI API Message">
                {
                    "model": "gpt-3.5-turbo",
                    "messages": [
                      {
                        "role": "system",
                        "content": "You are a helpful assistant."
                      },
                      {
                        "role": "user",
                        "content": "Tell me a story?"
                      }
                    ],
                    "stream": true
                }
                </textarea>
                <button onclick="sendMessage()">Send Chat Message</button>
                <textarea class="output" id="console-output" disabled></textarea>
            </div>
            <script>
                async function updateConfig() {
                    const input = document.getElementById('config-input').value;
                    try {
                        const { publicKey, iv, ciphertext } = await encryptMessage(input);
                        const response = await fetch('config', {
                            method: 'POST',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({ publicKey, iv, ciphertext })
                        });
                        const result = await response.json();
                        document.getElementById('console-output').value = JSON.stringify(result, null, 2);
                    } catch (e) {
                        document.getElementById('console-output').value = 'Error updating config: ' + e.message;
                    }
                }

                async function sendMessage() {
                    const input = document.getElementById('message-input').value;
                    try {
                        const response = await fetch('v1/chat/completions', {
                            method: 'POST',
                            headers: { 'Content-Type': 'application/json' },
                            body: input
                        });
                        let reply = '';
                        const outputArea = document.getElementById('console-output');
                        for await (const chunk of response.body) {
                            console.log('Received chunk:', chunk);
                            reply += new TextDecoder().decode(chunk);
                            outputArea.value = reply;
                            outputArea.scrollTop = outputArea.scrollHeight;
                        }
                    } catch (e) {
                        document.getElementById('console-output').value = 'Error sending message: ' + e.message;
                    }
                }

                function verify(k) {
                    // TODO: verify the signature or sgx quote to make sure the public key is from the app in TEE.
                }

                async function encryptMessage(message) {
                    const resp = await fetch('publicKey');
                    const signedPublicKey = JSON.parse(await resp.text());

                    verify(signedPublicKey);

                    const clientPrivateKey = await crypto.subtle.generateKey(
                        {
                            name: "ECDH",
                            namedCurve: "P-384",
                        },
                        false,
                        ["deriveKey"]
                    );

                    const clientPublicKey = clientPrivateKey.publicKey;
                    const exportedPublicKey = await crypto.subtle.exportKey("raw", clientPublicKey);
                    const derivedKey = await crypto.subtle.deriveKey(
                        {
                            name: "ECDH",
                            public: await crypto.subtle.importKey(
                                "raw",
                                fromHexString(signedPublicKey.ecdhPublicKey),
                                {
                                    name: "ECDH",
                                    namedCurve: "P-384",
                                },
                                false,
                                []
                            ),
                        },
                        clientPrivateKey.privateKey,
                        { name: "AES-GCM", length: 256 },
                        false,
                        ["encrypt"]
                    );

                    const iv = crypto.getRandomValues(new Uint8Array(12));
                    const encrypted = await crypto.subtle.encrypt(
                        { name: "AES-GCM", iv },
                        derivedKey,
                        new TextEncoder().encode(message)
                    );

                    return {
                        publicKey: toHexString(exportedPublicKey),
                        iv: toHexString(iv),
                        ciphertext: toHexString(new Uint8Array(encrypted))
                    };
                }

                function toHexString(buffer) {
                    return Array.prototype.map.call(new Uint8Array(buffer), x => ('00' + x.toString(16)).slice(-2)).join('');
                }

                function fromHexString(hexString) {
                    return new Uint8Array(hexString.match(/.{1,2}/g).map(byte => parseInt(byte, 16)));
                }
            </script>
        </body>
        </html>
    `;
}
