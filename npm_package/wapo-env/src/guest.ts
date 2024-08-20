import type { Hono } from "hono"
import type { BlankEnv, BlankSchema, Env, Schema } from 'hono/types'

export function handle<E extends Env = BlankEnv, S extends Schema = BlankSchema, BasePath extends string = '/'>(
  app: Hono<E, S, BasePath>,
) {
  return async function handler() {
    try {
      const data = JSON.parse(globalThis.scriptArgs?.[0])
      const req = new Request(data.url, {
        method: data.method,
        headers: data.headers,
        body: data.body,
      })
      const resp = await app.fetch(req)
      const headers: Record<string, string> = {}
      for (const [k, v] of resp.headers.entries()) {
        headers[k] = v
      }
      console.log('done', JSON.stringify(headers))
      // NOTE only suppport text for now.
      const body = await resp.text()
      globalThis.scriptOutput = JSON.stringify({ body, headers, status: resp.status })
    } catch (err) {
      // TODO error message formatting
      globalThis.scriptOutput = JSON.stringify({ body: (err as Error).message, headers: {}, status: 500 })
    }
  }
}
