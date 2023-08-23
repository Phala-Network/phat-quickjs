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
        constructor(id, receiver, head) {
            this.id = id;
            this.ok = ((head.status / 100) | 0) == 2;
            this.statusText = head.statusText;
            this.status = head.status;
            this.url = head["Location"] || head["location"] || "";
            this.headers = new Headers(head.headers);
            this.receiver = receiver;
            this.bodyUsed = false;
            this.type = "default";
            const chunks = [];
            receiver.recv = (cmd, data) => {
                if (cmd == "data") {
                    chunks.push(data);
                } else if (cmd == "end") {
                    const body = Sidevm.concatU8a(chunks);
                    receiver.resolve(body);
                } else {
                    receiver.reject(data);
                }
            }
        }
        async text() {
            return new TextDecoder().decode(await this.bytes());
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
                    Sidevm.close(reqId);
                }
            });
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
            const receiver = {
                recv: (cmd, data) => {
                    if (cmd == "head") {
                        if (redirect == "follow" && [301, 302, 307, 308].includes(data.status)) {
                            const location = data.headers['Location'];
                            if (location) {
                                g.fetch(location, options).then(resolve).catch(reject);
                                return;
                            }
                        }
                        resolve(new Response(reqId, receiver, data));
                    } else {
                        reject(data);
                    }
                },
                resolve: () => { },
                reject: () => { },
            }
            const reqId = Sidevm.httpRequest({
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
    g.Request = Request;
})(globalThis)