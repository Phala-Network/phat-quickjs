(function (g) {
    function toB(v) {
        if (!typeof v === 'string') {
            v = new Uint8Array(v);
        }
        return v;
    }
    g.pink = {
        invokeContract: (c) => {
            c.callee = toB(c.callee);
            c.input = toB(c.input);
            c.gasLimit = c.gasLimit || 0;
            c.value = c.value || 0;
            return __hostCall(0, c);
        },
        invokeContractDelegate: (c) => {
            c.codeHash = toB(c.codeHash);
            c.input = toB(c.input);
            return __hostCall(1, c);
        },
    };
}(globalThis))
