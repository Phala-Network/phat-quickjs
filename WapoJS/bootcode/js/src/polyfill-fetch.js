((g) => {
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
                !(bodyInit instanceof Uint8Array)
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
    g.Response = Response;
    g.Request = Request;
})(globalThis)
