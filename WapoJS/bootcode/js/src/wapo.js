(function (g) {
    function timerFn(hostFn) {
        return function (f, t) {
            t = t || 0;
            if (typeof f == 'string') {
                return hostFn(() => eval(f), t);
            }
            const args = Array.prototype.slice.call(arguments, 2);
            const callback = () => { f.apply(null, args); };
            return hostFn(callback, t);
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
    g.Wapo.concatU8a = concatU8a;
    g.setTimeout = timerFn(Wapo.setTimeout);
    g.setInterval = timerFn(Wapo.setInterval);
    g.clearTimeout = function (id) {
        if (id == null) return;
        return Wapo.close(id);
    };
    g.clearInterval = g.clearTimeout;
    g.setImmediate = function (f) {
        return Wapo.setTimeout(f, 0);
    };
    g.Wapo.inspect = function (...obj) {
        return Wapo.print(2, obj, {
            indent: '  ',
            depth: 5,
        });
    }
    g.console = {
        log(...args) {
            return Wapo.print(2, args);
        },
        info(...args) {
            return Wapo.print(2, args);
        },
        warn(...args) {
            return Wapo.print(3, args);
        },
        error(...args) {
            return Wapo.print(4, args);
        }
    }
    g.print = g.console.log;
    g.global = g;
    g.window = g;
    g.SCALE = Wapo.SCALE;
    g.process = {
        env: Wapo.env,
        exit: Wapo.exit,
        get argv() {
            return ["wapojs", "<eval>", ...scriptArgs];
        },
        version: "v0.9.0",
        nextTick: g.setImmediate,
    };
    g.performance = {
        now() {
            return new Date().getTime();
        }
    };
    g.localStorage = {
        _data: {},
        getItem(key) {
            return this._data[key];
        },
        setItem(key, value) {
            this._data[key] = value;
        },
        removeItem(key) {
            delete this._data[key];
        }
    };
}(globalThis))

export default {};
