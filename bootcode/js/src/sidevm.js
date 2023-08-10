(function (g) {
    function close(id) {
        __hostCall(1000, id);
    }
    function timerFn(call_id) {
        return function (f, t) {
            t = t || 0;
            if (typeof f == 'string') {
                return __hostCall(call_id, () => eval(f), t);
            }
            const args = Array.prototype.slice.call(arguments, 2);
            const callback = () => { f.apply(null, args); };
            return __hostCall(call_id, callback, t);
        }
    }
    function concatU8a(arrays) {
        const totalSize = arrays.reduce((acc, e) => acc + e.length, 0);
        const merged = new Uint8Array(totalSize);
        arrays.forEach((array, i, arrays) => {
            const offset = arrays.slice(0, i).reduce((acc, e) => acc + e.length, 0);
            merged.set(array, offset);
        });
        return merged;
    }
    g.sidevm = {
        close,
        concatU8a,
        httpRequest(cfg) {
            return __hostCall(1003, {
                method: "GET",
                headers: {},
                body: "0x",
                timeout: 10000,
                callback: () => { },
                ...cfg
            });
        },
        print(fd, ...args) {
            return __hostCall(1004, fd, ...args);
        },
        parseURL(url, base) {
            return __hostCall(1005, url, base);
        },
        parseURLParams(params) {
            return __hostCall(1006, params);
        }
    };
    g.setTimeout = timerFn(1001);
    g.setInterval = timerFn(1002);
    g.clearTimeout = close;
    g.clearInterval = close;
    g.console = {
        log(...args) {
            return sidevm.print(1, ...args);
        },
        error(...args) {
            return sidevm.print(2, ...args);
        },
        warn(...args) {
            return sidevm.print(2, ...args);
        }
    }
    g.print = g.console.log;
}(globalThis))

export default {};
