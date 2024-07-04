((g) => {
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
                    timeoutMs: options.timeoutMs || 10000,
                    body: options.body || "",
                },
                (cmd, data) => receiver.recv(cmd, data),
            );
        });
    };
})(globalThis)
