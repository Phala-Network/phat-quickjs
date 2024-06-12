(function (g) {
    g.self = g;
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
            case "http2": return require("http2");
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
}(globalThis))