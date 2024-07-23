var scope = globalThis;
var apply = Function.prototype.apply;

exports.setTimeout = function () {
    const createFn = () => apply.call(setTimeout, scope, arguments);
    return new Timeout(createFn, clearTimeout);
};
exports.setInterval = function () {
    const createFn = () => apply.call(setInterval, scope, arguments);
    return new Timeout(createFn, clearInterval);
};
exports.clearTimeout =
    exports.clearInterval = function (timeout) {
        if (timeout) {
            timeout.close();
        }
    };

function Timeout(createFn, clearFn) {
    this._id = createFn();
    this._createFn = createFn;
    this._clearFn = clearFn;
}
Timeout.prototype.unref = Timeout.prototype.ref = function () { };
Timeout.prototype.close = function () {
    this._clearFn.call(scope, this._id);
};
Timeout.prototype.refresh = function () {
    this._clearFn.call(scope, this._id);
    this._id = this._createFn();
};

// Does not start the time, just sets up the members needed.
exports.enroll = function (item, msecs) {
    clearTimeout(item._idleTimeoutId);
    item._idleTimeout = msecs;
};

exports.unenroll = function (item) {
    clearTimeout(item._idleTimeoutId);
    item._idleTimeout = -1;
};

exports._unrefActive = exports.active = function (item) {
    clearTimeout(item._idleTimeoutId);

    var msecs = item._idleTimeout;
    if (msecs >= 0) {
        item._idleTimeoutId = setTimeout(function onTimeout() {
            if (item._onTimeout)
                item._onTimeout();
        }, msecs);
    }
};

// On some exotic environments, it's not clear which object `setimmediate` was
// able to install onto.  Search each possibility in the same order as the
// `setimmediate` library.
exports.setImmediate = (typeof self !== "undefined" && self.setImmediate) ||
    (typeof global !== "undefined" && global.setImmediate) ||
    (this && this.setImmediate);
exports.clearImmediate = (typeof self !== "undefined" && self.clearImmediate) ||
    (typeof global !== "undefined" && global.clearImmediate) ||
    (this && this.clearImmediate);