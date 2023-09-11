(function (g) {
    g.console = {
        log(...args) {
            Pink.print(0, 2, args);
        },
        info(...args) {
            Pink.print(0, 2, args);
        },
        warn(...args) {
            Pink.print(0, 3, args);
        },
        error(...args) {
            Pink.print(0, 4, args);
        }
    };
    Pink.inspect = function (...args) {
        Pink.print(5, 2, args);
    };
}(globalThis))
export default {};
