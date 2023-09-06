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
        invokeContractDelegate(c) {
            return __hostCall(1, {
                codeHash: toB(c.codeHash),
                selector: c.selector,
                input: toB(c.input),
                allowReentry: c.allowReentry || false,
            });
        },
        httpRequest(c) {
            return __hostCall(2, c);
        },
        batchHttpRequest(c, t) {
            return __hostCall(3, c, t || 10000);
        },
        hash(alg, m) {
            return __hostCall(5, alg, m);
        },
    };
}(globalThis))
export default {};
