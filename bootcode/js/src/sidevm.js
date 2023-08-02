(function (g) {
    function toB(v) {
        if (typeof v != 'string') {
            v = new Uint8Array(v);
        }
        return v;
    }
    g.setTimeout = function (f, t) {
        const callback_args = Array.prototype.slice.call(arguments, 2);
        const callback = () => { f.apply(null, callback_args); };
        return __hostCall(1001, callback, t || 0);
    }
    g.clearTimeout = function (id) {
        return __hostCall(1002, id);
    }
    g.setInterval = function (f, t) {
        const callback_args = Array.prototype.slice.call(arguments, 2);
        const callback = () => { f.apply(null, callback_args); };
        return __hostCall(1003, callback, t || 0);
    }
    g.clearInterval = function (id) {
        return __hostCall(1004, id);
    }
}(globalThis))
export default {};
