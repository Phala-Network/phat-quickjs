((g) => {
    class Request {
        constructor(input, init = {}) {
            if (input instanceof Request) {
                this.url = input.url;
                this.method = input.method;
                this.headers = new Headers(input.headers);
                this.body = input.body;
            } else {
                this.url = input;
                this.method = init.method || 'GET';
                this.headers = new Headers(init.headers);
                this.body = init.body || null;
            }
            this.cache = init.cache || 'default';
            this.redirect = init.redirect || 'follow';
            this.referrer = init.referrer || 'about:client';
        }
    }

    class Response {
        constructor(bodyInit = null, options = {}) {
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
                this.setBody(bodyInit);
            }
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
        bytes() {
            return new Promise((resolve, reject) => {
                const reader = this.body.getReader();
                const chunks = [];
                reader.read().then(function processText({ done, value }) {
                    if (done) {
                        resolve(Wapo.concatU8a(chunks));
                    } else {
                        chunks.push(value);
                        reader.read().then(processText);
                    }
                }).catch(reject);
            });
        }
        get body() {
            return this._body ? this._body : this.createBodyStream();
        }

        setBody(bodyInit) {
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
            }
        }

        createBodyStream() {
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

    g.fetch = (resource, options) => {
        var url;
        if (typeof resource == "string") {
            url = resource;
        } else if (resource instanceof Request) {
            url = resource.url;
            options = {
                ...resource,
                ...options || {},
            }
        } else {
            url = resource.toString();
        }
        options = options || {};
        const redirect = options.redirect || "follow";
        return new Promise((resolve, reject) => {
            const request = {
                url,
                method: options.method || "GET",
                headers: options.headers || {},
                timeout: options.timeout,
                body: options.body || "",
            };
            if (request.headers instanceof Headers) {
                request.headers = Object.fromEntries(options.headers.entries());
            }
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
