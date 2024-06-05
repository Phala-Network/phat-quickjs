
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