
((g) => {
    if (!g.AbortSignal) {
        return;
    }
    if (AbortSignal.timeout === undefined) {
        AbortSignal.timeout = function (ms) {
            const signal = new AbortSignal();
            setTimeout(() => {
                signal.dispatchEvent(new Event("abort"));
            }, ms);
            return signal;
        };
        return;
    }
})(globalThis);