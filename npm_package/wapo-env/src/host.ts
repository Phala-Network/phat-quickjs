import type { IncomingRequest, HttpConfig, HttpsConfig } from "./index"
import type { Hono } from "hono"
import type { BlankEnv, BlankSchema, Env, Schema } from 'hono/types'

export class WapoRequest extends Request {
  opaqueRequest: IncomingRequest;

  constructor(req: IncomingRequest, init?: RequestInit) {
    const headers = new Headers(req.headers);
    const host = headers.get("host") || "localhost";
    const url = `https://${host}${req.url}`;
    const body = new ReadableStream({
      start(controller) {
        Wapo.streamOpenRead(req.opaqueInputStream, (cmd, data) => {
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
              throw new Error(`unknown cmd: ${cmd}`);
          }
        });
      }
    })

    super(url, { ...(init || {}), ...req, headers, body })

    this.opaqueRequest = req
  }
}

export function sendResponse(response: Response, req: IncomingRequest) {
  const writer = Wapo.streamOpenWrite(req.opaqueOutputStream)

  const stream = new WritableStream({
    write(chunk) {
      return new Promise<void>((resolve, reject) => {
        Wapo.streamWriteChunk(writer, chunk, (suc, err) => {
          if (suc) {
            resolve()
          } else {
            reject(err)
          }
        })
      })
    },
    close() {
      Wapo.streamClose(writer)
    }
  })

  Wapo.httpsSendResponseHead(req.opaqueResponseTx, {
    status: response.status,
    headers: Array.from(response.headers.entries()),
  });

  response.body?.pipeTo(stream)
}

export type handleOptions = Omit<HttpsConfig, "serverName"> & { serverName?: string } | HttpConfig

export function handle<E extends Env = BlankEnv, S extends Schema = BlankSchema, BasePath extends string = '/'>(
  app: Hono<E, S, BasePath>,
  opts: handleOptions
) {
  const isHttps = !("address" in opts)
  let serverName = isHttps ? 'localhost' : opts.address
  // NOTE: make linter happy.
  if (!("address" in opts) && !opts.serverName) {
    opts.serverName = serverName
  }
  return function handler() {
    Wapo.httpsListen(
      opts as (HttpsConfig | HttpConfig),
      async req => {
        try {
          const request = new WapoRequest(req)
          if (isHttps && request.headers.get('host') !== serverName) {
            sendResponse(new Response("Upgrade Required", { status: 426 }), req)
          } else {
            const response = await app.fetch(request)
            sendResponse(response, req)
          }
        } catch (error) {
          sendResponse(new Response("Internal Server Error", { status: 500 }), req)
        }
      }
    );
  };   
}
