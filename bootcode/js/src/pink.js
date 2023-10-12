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
}(globalThis))
export default {};
