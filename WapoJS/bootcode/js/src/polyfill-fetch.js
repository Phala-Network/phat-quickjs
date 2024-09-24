((g) => {
    class Headers {
        constructor(init = {}) {
            this._headers = new Map();
            if (init instanceof Headers) {
                init.forEach((value, name) => this.append(name, value));
            } else if (Array.isArray(init)) {
                init.forEach(([name, value]) => this.append(name, value));
            } else if (typeof init === 'object') {
                Object.entries(init).forEach(([name, value]) => this.append(name, value));
            }
        }

        append(name, value) {
            name = name.toLowerCase();
            if (!this._headers.has(name)) {
                this._headers.set(name, []);
            }
            this._headers.get(name).push(String(value));
        }

        delete(name) {
            this._headers.delete(name.toLowerCase());
        }

        get(name) {
            const values = this._headers.get(name.toLowerCase());
            return values ? values[0] : null;
        }

        has(name) {
            return this._headers.has(name.toLowerCase());
        }

        set(name, value) {
            this._headers.set(name.toLowerCase(), [String(value)]);
        }

        forEach(callback, thisArg) {
            for (const [name, values] of this._headers) {
                callback.call(thisArg, values.join(', '), name, this);
            }
        }

        *entries() {
            for (const [name, values] of this._headers) {
                yield [name, values.join(', ')];
            }
        }

        getSetCookie() {
            return this._headers.get('set-cookie') || [];
        }

        *keys() {
            for (const name of this._headers.keys()) {
                yield name;
            }
        }

        *values() {
            for (const [, values] of this._headers) {
                yield values.join(', ');
            }
        }

        toString() {
            return '[object Headers]';
        }
    }

    class FormData {
        constructor() {
            this._entries = [];
        }

        append(name, value, filename) {
            this._entries.push([name, value, filename]);
        }

        delete(name) {
            this._entries = this._entries.filter(entry => entry[0] !== name);
        }

        get(name) {
            const entry = this._entries.find(entry => entry[0] === name);
            return entry ? entry[1] : null;
        }

        getAll(name) {
            return this._entries.filter(entry => entry[0] === name).map(entry => entry[1]);
        }

        has(name) {
            return this._entries.some(entry => entry[0] === name);
        }

        set(name, value, filename) {
            const index = this._entries.findIndex(entry => entry[0] === name);
            if (index !== -1) {
                this._entries[index] = [name, value, filename];
            } else {
                this._entries.push([name, value, filename]);
            }
        }

        *[Symbol.iterator]() {
            for (const entry of this._entries) {
                yield [entry[0], entry[1]];
            }
        }

        *entries() {
            for (const entry of this._entries) {
                yield [entry[0], entry[1]];
            }
        }

        *keys() {
            for (const entry of this._entries) {
                yield entry[0];
            }
        }

        *values() {
            for (const entry of this._entries) {
                yield entry[1];
            }
        }

        forEach(callback, thisArg) {
            for (const [name, value] of this) {
                callback.call(thisArg, value, name, this);
            }
        }

        toString() {
            return '[object FormData]';
        }
    }

    function consumed(body) {
        if (body._noBody) return
        if (body.bodyUsed) {
            throw new TypeError('Already read')
        }
        body.bodyUsed = true
    }

    class WithBody {
        constructor() {
            this.bodyUsed = false
        }
        async text() {
            return Wapo.utf8Decode(await this.bytes());
        }
        async json() {
            return JSON.parse(await this.text());
        }
        async blob() {
            return new Blob([await this.bytes()]);
        }
        async arrayBuffer() {
            return (await this.bytes()).buffer;
        }
        async bytes() {
            const reader = this.body.getReader();
            const chunks = [];
            while (true) {
                const { done, value } = await reader.read();
                if (value) {
                    chunks.push(value);
                }
                if (done) {
                    return Wapo.concatU8a(chunks);
                }
            }
        }
        async formData() {
            const formData = new FormData();
            const text = await this.text();
            const pairs = text.split('&');
            for (const pair of pairs) {
                const [name, value] = pair.split('=');
                formData.append(decodeURIComponent(name), decodeURIComponent(value));
            }
            return formData;
        }
        get body() {
            consumed(this);
            return this._body ? this._body : this.createBodyStream();
        }
        _initBody(bodyInit) {
            if (typeof bodyInit === 'string') {
                this._body = new ReadableStream({
                    start(controller) {
                        controller.enqueue(new TextEncoder().encode(bodyInit));
                        controller.close();
                    }
                });
            } else if (bodyInit instanceof Blob) {
                this._body = new ReadableStream({
                    start(controller) {
                        bodyInit.arrayBuffer().then(buffer => {
                            controller.enqueue(new Uint8Array(buffer));
                            controller.close();
                        });
                    }
                });
            } else if (bodyInit instanceof ArrayBuffer) {
                this._body = new ReadableStream({
                    start(controller) {
                        controller.enqueue(new Uint8Array(bodyInit));
                        controller.close();
                    }
                });
            } else if (bodyInit instanceof Uint8Array) {
                this._body = new ReadableStream({
                    start(controller) {
                        controller.enqueue(bodyInit);
                        controller.close();
                    }
                });
            } else if (bodyInit instanceof ReadableStream) {
                this._body = bodyInit;
            // The case of subclass of String
            } else if (bodyInit instanceof String && typeof bodyInit === 'object' && typeof bodyInit.toString === 'function') {
                this._body = new ReadableStream({
                    start(controller) {
                        controller.enqueue(new TextEncoder().encode(bodyInit.toString()));
                        controller.close();
                    }
                });
            }
        }
        createBodyStream() {
            if (!this._opaqueBodyStream) {
                this._body = new ReadableStream({
                    start(controller) {
                        controller.close();
                    }
                });
                return this._body;
            }
            const anchor = {};
            const opaqueBodyStream = this._opaqueBodyStream;
            this._body = new ReadableStream({
                start(controller) {
                    anchor.reqId = Wapo.streamOpenRead(opaqueBodyStream, (cmd, data) => {
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
                },
                cancel() {
                    if (anchor.reqId) {
                        Wapo.close(anchor.reqId);
                    }
                }
            });
            return this._body;
        }
    }

    const methods = ['CONNECT', 'DELETE', 'GET', 'HEAD', 'OPTIONS', 'PATCH', 'POST', 'PUT', 'TRACE']
    function normalizeMethod(method) {
        const upcased = method.toUpperCase()
        return methods.indexOf(upcased) > -1 ? upcased : method
    }

    class Request extends WithBody {
        constructor(input, options = {}) {
            super()
            options = options || {}
            let body = options.body

            if (input instanceof Request) {
                if (input.bodyUsed) {
                    throw new TypeError('Already read')
                }
                this.url = input.url
                this.credentials = input.credentials
                if (!options.headers) {
                    this.headers = new Headers(input.headers)
                }
                this.method = input.method
                this.mode = input.mode
                this.signal = input.signal
                if (!body && input._bodyInit != null) {
                    body = input._bodyInit
                    input.bodyUsed = true
                }
                this.redirect = input.redirect;
                this.referrer = input.referrer;
            } else {
                this.url = String(input)
            }

            this.redirect = options.redirect || this.redirect || 'follow';
            this.referrer = options.referrer || this.referrer || 'about:client';
            this.credentials = options.credentials || this.credentials || 'same-origin'
            if (options.headers || !this.headers) {
                this.headers = new Headers(options.headers)
            }
            this.method = normalizeMethod(options.method || this.method || 'GET')
            this.mode = options.mode || this.mode || null
            this.signal = options.signal || this.signal || (function () {
                if ('AbortController' in g) {
                    var ctrl = new AbortController();
                    return ctrl.signal;
                }
            }());
            this._initBody(body)
            if (this.method === 'GET' || this.method === 'HEAD') {
                if (options.cache === 'no-store' || options.cache === 'no-cache') {
                    // Search for a '_' parameter in the query string
                    var reParamSearch = /([?&])_=[^&]*/
                    if (reParamSearch.test(this.url)) {
                        // If it already exists then set the value with the current time
                        this.url = this.url.replace(reParamSearch, '$1_=' + new Date().getTime())
                    } else {
                        // Otherwise add a new '_' parameter to the end with the current time
                        var reQueryString = /\?/
                        this.url += (reQueryString.test(this.url) ? '&' : '?') + '_=' + new Date().getTime()
                    }
                }
            }
        }
    }

    class Response extends WithBody {
        constructor(bodyInit = null, options = {}) {
            super()
            if (
                bodyInit !== null &&
                typeof bodyInit !== 'string' &&
                !(bodyInit instanceof Blob) &&
                !(bodyInit instanceof ArrayBuffer) &&
                !(bodyInit instanceof Uint8Array) &&
                !(bodyInit instanceof String)
            ) {
                throw new TypeError('Unsupported bodyInit type');
            }

            this.id = options.id || null;
            this.ok = (options.status / 100 | 0) == 2;
            this.statusText = options.statusText || '';
            this.status = options.status || 200;
            this.url = options.url || '';
            this.headers = new Headers(options.headers || {});
            this._opaqueBodyStream = options.opaqueBodyStream || null;
            this.bodyUsed = false;
            this.type = "default";
            if (bodyInit !== null) {
                this._initBody(bodyInit);
            }
        }
    }

    g.fetch = async (resource, options) => {
        return new Promise(async (resolve, reject) => {
            const r = new Request(resource, options);
            const request = {
                url: r.url,
                method: r.method,
                headers: Object.fromEntries(r.headers.entries()),
                body: await r.bytes(),
            };
            const redirect = r.redirect;
            Wapo.httpRequest(request,
                (cmd, data) => {
                    if (cmd == "head") {
                        if (redirect == "follow" && [301, 302, 307, 308].includes(data.status)) {
                            const headers = new Headers(data.headers);
                            const location = headers.get('Location');
                            if (location) {
                                let url;
                                if (location.startsWith("http")) {
                                    url = location;
                                } else if (location.startsWith("//")) {
                                    const base = new URL(request.url);
                                    url = base.protocol + location;
                                } else {
                                    url = new URL(location, request.url).href;
                                }
                                g.fetch(url, options).then(resolve).catch(reject);
                                return;
                            }
                        }
                        resolve(new Response(null, data));
                    } else {
                        reject(data);
                    }
                }
            );
        });
    };

    g.Headers = Headers;
    g.FormData = FormData;
    g.Response = Response;
    g.Request = Request;
})(globalThis)
