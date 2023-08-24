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
    g.Sidevm.concatU8a = concatU8a;
    g.setTimeout = timerFn(Sidevm.setTimeout);
    g.setInterval = timerFn(Sidevm.setInterval);
    g.clearTimeout = Sidevm.close;
    g.clearInterval = Sidevm.close;
    g.Sidevm.inspect = function (...obj) {
        return Sidevm.print(3, 1, obj);
    }
    g.console = {
        log(...args) {
            return Sidevm.print(0, 1, args);
        },
        error(...args) {
            return Sidevm.print(0, 2, args);
        },
        warn(...args) {
            return Sidevm.print(0, 2, args);
        }
    }
    g.print = g.console.log;
}(globalThis))

export default {};
