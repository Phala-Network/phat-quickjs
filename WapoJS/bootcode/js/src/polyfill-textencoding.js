'use strict';
(function (g) {
    class TextEncoder {
        encode(str) {
            if (str == null) {
                return new Uint8Array();
            }
            return Wapo.utf8Encode(str);
        }
        encodeInto(str, dest) {
            if (str == null) {
                return {
                    read: 0,
                    written: 0
                };
            }
            const bytes = this.encode(str);
            const length = Math.min(dest.length, bytes.length);
            dest.set(bytes.subarray(0, length));
            return {
                read: str.length,
                written: length
            };
        }
    }
    g.TextEncoder = TextEncoder;
})(globalThis);
(function (g) {
    class TextDecoder {
        constructor(encoding = 'utf-8') {
            const normalizedEncoding = encoding.toLowerCase();
            if (normalizedEncoding !== 'utf-8' && normalizedEncoding !== 'utf8') {
                throw new TypeError('Unsupported text encoding: ' + encoding);
            }
        }
        decode(bytes) {
            if (bytes == null) {
                return '';
            }
            return Wapo.utf8Decode(bytes);
        }
    }
    g.TextDecoder = TextDecoder;
}
)(globalThis);
export default {};