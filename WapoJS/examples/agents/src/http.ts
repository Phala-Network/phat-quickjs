import { ReadableStreamHandle, TlsConfig, WriteableStreamHandle } from "@phala/wapo-env";

export function listen(config: TlsConfig, handler: (req: Request) => Response | Promise<Response>) {
    Wapo.httpsListen(config, async req => {
        const headers = new Headers(req.headers);
        const host = headers.get("host") || "localhost";
        const url = `https://${host}${req.url}`;
        const request = new Request(url, {
            ...req,
            body: toReadableStream(req.opaqueInputStream),
        });
        const response = await handler(request);
        const resposneStream = toWritableStream(req.opaqueOutputStream);
        console.log(`${req.method} ${req.url}: ${response.status} ${response.statusText}`);
        Wapo.httpsSendResponseHead(req.opaqueResponseTx, {
            status: response.status,
            headers: Array.from(response.headers.entries()),
        });
        response.body.pipeTo(resposneStream);
    });
}

function toReadableStream(body: ReadableStreamHandle): ReadableStream<Uint8Array> {
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

function toWritableStream(streamId: WriteableStreamHandle): WritableStream<Uint8Array> {
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
