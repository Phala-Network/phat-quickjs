(function (g) {
    const hostCall = g.__hostCall;
    function toB(v) {
        if (typeof v != 'string') {
            v = new Uint8Array(v);
        }
        return v;
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
        }
    };
    g.fetch = (url, options) => {
        options = options || {};
        return new Promise((resolve, reject) => {
            function makeResponse(head) {
                const next1 = {
                    resolve: (data) => { },
                    reject: (data) => { },
                }
                const response = {
                    ok: ((head.status / 100) | 0) == 2,
                    statusText: head.statusText,
                    status: head.status,
                    url,
                    text: () => {
                        return new Promise((resolve, reject) => {
                            next1.resolve = (data) => resolve(new TextDecoder().decode(data));
                            next1.reject = reject;
                        });
                    },
                    json: () => {
                        return new Promise((resolve, reject) => {
                            next1.resolve = (data) => resolve(JSON.parse(new TextDecoder().decode(data)));
                            next1.reject = reject;
                        });
                    },
                    blob: () => {
                        return new Promise((resolve, reject) => {
                            next1.resolve = resolve;
                            next1.reject = reject;
                        });
                    },
                    headers: {
                        keys: () => head.headers.keys(),
                        entries: () => head.headers.entries(),
                        get: (n) => head.headers.get(n),
                        has: (n) => head.headers.has(n),
                    },
                };
                const chunks = [];
                next0.callback = (cmd, data) => {
                    if (cmd == "data") {
                        chunks.push(data);
                    } else if (cmd == "end") {
                        next1.resolve(mergeUint8Arrays(...chunks));
                    } else {
                        next1.reject(data);
                    }
                };
                return response;
            };
            const next0 = {
                callback: (cmd, data) => {
                    if (cmd == "head") {
                        resolve(makeResponse(data));
                    } else {
                        reject(data);
                    }
                },
            }
            g.sidevm.httpRequest({
                url,
                method: options.method || "GET",
                headers: options.headers || {},
                callback: (cmd, data) => {
                    next0.callback(cmd, data);
                },
            });
        });
    };
}(globalThis))
export default {};
