
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
    }

    if (AbortSignal.prototype.throwIfAborted === undefined) {
        AbortSignal.prototype.throwIfAborted = function () {
            if (this.aborted) {
                throw new DOMException("The operation was aborted.", "AbortError");
            }
        };
    }
})(globalThis);