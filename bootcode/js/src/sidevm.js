(function (g) {
    const ecall = g.__hostCall;
    function toB(v) {
        if (typeof v != 'string') {
            v = new Uint8Array(v);
        }
        return v;
    }
    function timerFn(call_id) {
        return function (f, t) {
            t = t || 0;
            if (typeof f == 'string') {
                return ecall(call_id, () => eval(f), t);
            }
            const args = Array.prototype.slice.call(arguments, 2);
            const callback = () => { f.apply(null, args); };
            return ecall(call_id, callback, t);
        }
    }
    g.setTimeout = timerFn(1001);
    g.clearTimeout = function (id) {
        return ecall(1002, id);
    };
    g.setInterval = timerFn(1003);
    g.clearInterval = function (id) {
        return ecall(1004, id);
    }
}(globalThis))
export default {};
