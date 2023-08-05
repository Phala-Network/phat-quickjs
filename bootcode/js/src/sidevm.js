(function (g) {
    const hostCall = g.__hostCall;
    function toB(v) {
        if (typeof v != 'string') {
            v = new Uint8Array(v);
        }
        return v;
    }
    function close(id) {
        hostCall(1000, id);
    }
    function timerFn(call_id) {
        return function (f, t) {
            t = t || 0;
            if (typeof f == 'string') {
                return hostCall(call_id, () => eval(f), t);
            }
            const args = Array.prototype.slice.call(arguments, 2);
            const callback = () => { f.apply(null, args); };
            return hostCall(call_id, callback, t);
        }
    }
    g.setTimeout = timerFn(1001);
    g.setInterval = timerFn(1002);
    g.clearTimeout = close;
    g.clearInterval = close;
    g.sidevm = {
        close,
        httpRequest(cfg) {
            const id = hostCall(1003, {
                method: "GET",
                headers: {},
                body: "0x",
                timeout: 10000,
                callback: () => { },
                ...cfg
            });
            return id;
        },
    };
    implFetch(g);

    function implFetch(g) {
        class Headers {
            constructor(headers) {
                this._headers = headers;
            }
            keys() {
                return this._headers.keys();
            }
            entries() {
                return this._headers.entries();
            }
            get(n) {
                return this._headers.get(n);
            }
            has(n) {
                return this._headers.has(n);
            }
        }

        class Response {
            constructor(url, id, receiver, head) {
                this.id = id;
                this.ok = ((head.status / 100) | 0) == 2;
                this.statusText = head.statusText;
                this.status = head.status;
                this.url = url;
                this.headers = new Headers(head.headers);
                this.receiver = receiver;
                this.bodyUsed = false;
                this.type = "default";
                const chunks = [];
                receiver.recv = (cmd, data) => {
                    if (cmd == "data") {
                        chunks.push(data);
                    } else if (cmd == "end") {
                        const body = mergeUint8Arrays(...chunks);
                        receiver.resolve(body);
                    } else {
                        receiver.reject(data);
                    }
                }
            }
            async text() {
                const data = await this.bytes();
                return new TextDecoder().decode(data);
            }
            async json() {
                const data = await this.text();
                return JSON.parse(data);
            }
            async blob() {
                const data = await this.bytes();
                return new Blob([data]);
            }
            async arrayBuffer() {
                const data = await this.bytes();
                return data.buffer;
            }
            bytes() {
                const r = this.receiver;
                return new Promise((resolve, reject) => {
                    r.resolve = resolve
                    r.reject = reject;
                });
            }
            get body() {
                const r = this.receiver;
                const reqId = this.id;
                return new ReadableStream({
                    start(controller) {
                        r.recv = (cmd, data) => {
                            if (cmd == "data") {
                                controller.enqueue(data);
                            } else if (cmd == "end") {
                                controller.close();
                            } else {
                                controller.error(data);
                            }
                        }
                    },
                    cancel() {
                        sidevm.close(reqId);
                    }
                });
            }
        }

        g.fetch = (url, options) => {
            options = options || {};
            return new Promise((resolve, reject) => {
                const receiver = {
                    recv: (cmd, data) => {
                        if (cmd == "head") {
                            resolve(new Response(url, reqId, receiver, data));
                        } else {
                            reject(data);
                        }
                    },
                    resolve: () => { },
                    reject: () => { },
                }
                const reqId = sidevm.httpRequest({
                    url,
                    method: options.method || "GET",
                    headers: options.headers || {},
                    timeout: options.timeout || 10000,
                    body: options.body || "0x",
                    callback: (cmd, data) => receiver.recv(cmd, data),
                });
            });
        };
        g.Response = Response;
        g.Headers = Headers;
    }
    function mergeUint8Arrays(...arrays) {
        const totalSize = arrays.reduce((acc, e) => acc + e.length, 0);
        const merged = new Uint8Array(totalSize);
        arrays.forEach((array, i, arrays) => {
            const offset = arrays.slice(0, i).reduce((acc, e) => acc + e.length, 0);
            merged.set(array, offset);
        });
        return merged;
    }
}(globalThis))

export default {};
