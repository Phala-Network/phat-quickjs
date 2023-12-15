(function (g) {
    g.console = {
        log(...args) {
            Pink.print(2, args);
        },
        info(...args) {
            Pink.print(2, args);
        },
        warn(...args) {
            Pink.print(3, args);
        },
        error(...args) {
            Pink.print(4, args);
        }
    };
    Pink.inspect = function (...args) {
        Pink.print(2, args, {
            depth: 5,
            indent: '  ',
        });
    };
}(globalThis));

(function(g){
    // Polyfill for TextEncoder/TextDecoder
    function TextEncoder() {
    }
    TextEncoder.prototype.encode = function (s) {
        return Pink.utf8Encode(s);
    };
    TextEncoder.prototype.encodeInto = function (s, u8a) {
        return Pink.utf8EncodeInto(s, u8a);
    };
    function TextDecoder() {
    }
    TextDecoder.prototype.decode = function (octets) {
        return Pink.utf8Decode(octets);
    };
    g.TextDecoder = TextDecoder;
    g.TextEncoder = TextEncoder;
}(globalThis));
export default {};
