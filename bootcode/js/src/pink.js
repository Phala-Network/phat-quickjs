(function (g) {
    function toB(v) {
        if (typeof v != 'string') {
            v = new Uint8Array(v);
        }
        return v;
    }
    g.pink = {
        invokeContract(c) {
            return __hostCall(0, {
                callee: toB(c.callee),
                selector: c.selector,
                input: toB(c.input),
                gasLimit: c.gasLimit || 0,
                value: c.value || 0,
                allowReentry: c.allowReentry || false,
            });
        },
        httpRequest(c) {
            return __hostCall(2, {
                ...c,
                method: c.method || "GET",
            });
        },
    };
    if (g.scriptArgs) {
        g.process = { argv: ["/node", ...g.scriptArgs] }
    }
}(globalThis))
export default {};
