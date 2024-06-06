
((g) => {
    g.Event = class Event {
        constructor(type, eventInitDict = {}) {
            this.type = type;
            this.bubbles = eventInitDict.bubbles || false;
            this.cancelable = eventInitDict.cancelable || false;
            this.composed = eventInitDict.composed || false;
        }
        stopPropagation() { }
        stopImmediatePropagation() { }
        preventDefault() { }
    };

    g.EventTarget = class EventTarget {
        constructor() {
            this._listeners = {};
        }
        addEventListener(type, callback) {
            if (!this._listeners[type]) {
                this._listeners[type] = [];
            }
            this._listeners[type].push(callback);
        }
        removeEventListener(type, callback) {
            if (!this._listeners[type]) {
                return;
            }
            this._listeners[type] = this._listeners[type].filter((cb) => cb !== callback);
        }
        dispatchEvent(event) {
            if (!this._listeners[event.type]) {
                return;
            }
            this._listeners[event.type].forEach((cb) => cb(event));
        }
    }

    g.DOMException = class DOMException extends Error {
        constructor(message, name) {
            super(message);
            this.name = name || 'DOMException';
            this.message = message;
        }
        [Symbol.toStringTag] = 'DOMException';
        toString() {
            return this.name + ': ' + this.message;
        }
    }
})(globalThis);