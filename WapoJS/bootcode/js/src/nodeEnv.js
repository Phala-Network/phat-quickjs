(function (g) {
    g.self = g;
    g.Buffer = require("buffer").Buffer;
    function nodeRequire(id) {
        if (id.startsWith("node:")) id = id.slice(5);
        switch (id) {
            case "assert": return require("assert");
            case "assert/strict": return require("assert/strict");
            case "async_hooks": return require("async_hooks");
            case "buffer": return require("buffer");
            case "child_process": return require("child_process");
            case "cluster": return require("cluster");
            case "console": return require("console");
            case "constants": return require("constants");
            case "crypto": return require("crypto");
            case "diagnostics_channel": return require("diagnostics_channel");
            case "dns": return require("dns");
            case "domain": return require("domain");
            case "events": return require("events");
            case "fs": return require("fs");
            case "fs/promises": return require("fs/promises");
            case "http": return require("http");
            case "https": return require("https");
            case "module": return require("module");
            case "net": return require("net");
            case "os": return require("os");
            case "path": return require("path");
            case "perf_hooks": return require("perf_hooks");
            case "process": return require("process");
            case "punycode": return require("punycode");
            case "querystring": return require("querystring");
            case "readline": return require("readline");
            case "repl": return require("repl");
            case "stream": return require("stream");
            case "string_decoder": return require("string_decoder");
            case "sys": return require("sys");
            case "timers": return require("timers");
            case "timers/promises": return require("timers/promises");
            case "tls": return require("tls");
            case "tty": return require("tty");
            case "url": return require("url");
            case "util": return require("util");
            case "v8": return require("v8");
            case "vm": return require("vm");
            case "wasi": return require("wasi");
            case "worker_threads": return require("worker_threads");
            case "zlib": return require("zlib");
        }
    }
    g.require = nodeRequire;
    g.__dirname = "/";
    g.__filename = "index.js";
    g.navigator = {
        deviceMemory: 8,
        hardwareConcurrency: 1,
        language: "en-US",
        userActivation: false,
        userAgent: "Node.js",
    };
    g.name = "nodejs";
    g.location = {
        protocol: "https:",
        pathname: "/",
        search: "",
        hash: "",
        href: "https://localhost/",
        origin: "https://localhost",
        hostname: "localhost",
        host: "localhost",
    };
    patchHttpAgent();
    patchWebCrypto(g);
    patchTextCoders(g);

    function patchHttpAgent() {
        const http = nodeRequire("http");
        http.Agent = class Agent {
            constructor(options) {
                this.options = options || {};
            }
            on() { }
        }
        http.Agent.defaultMaxSockets = 4
        http.globalAgent = new http.Agent()
    }

    function patchWebCrypto(g) {
        nodeRequire("node:crypto").webcrypto = g.crypto;
    }

    function patchTextCoders(g) {
        const util = nodeRequire("node:util");
        util.TextEncoder = g.TextEncoder;
        util.TextDecoder = g.TextDecoder;
    }
}(globalThis))
