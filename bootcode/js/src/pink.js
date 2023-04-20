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
            });
        },
        invokeContractDelegate(c) {
            return __hostCall(1, {
                codeHash: toB(c.codeHash),
                selector: c.selector,
                input: toB(c.input),
            });
        },
        httpRequest(c) {
            return __hostCall(2, {
                ...c,
                method: c.method || "GET",
            });
        },
    };
    g.process = { argv: ["/node", ...scriptArgs] }
}(globalThis))
export default {};