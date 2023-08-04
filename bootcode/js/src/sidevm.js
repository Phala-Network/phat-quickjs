(function (g) {
    const hostCall = g.__hostCall;
    function toB(v) {
        if (typeof v != 'string') {
            v = new Uint8Array(v);
        }
        return v;
    }
    function close(id) {
        hostCall(1000, id);
    }
    function timerFn(call_id) {
        return function (f, t) {
            t = t || 0;
            if (typeof f == 'string') {
                return hostCall(call_id, () => eval(f), t);
            }
            const args = Array.prototype.slice.call(arguments, 2);
            const callback = () => { f.apply(null, args); };
            return hostCall(call_id, callback, t);
        }
    }
    g.setTimeout = timerFn(1001);
    g.setInterval = timerFn(1002);
    g.clearTimeout = close;
    g.clearInterval = close;
}(globalThis))
export default {};
