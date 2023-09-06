(function (g) {
    g.console = {
        log(...args) {
            pink.log(1, args);
        },
        info(...args) {
            pink.log(2, args);
        },
        warn(...args) {
            pink.log(3, args);
        },
        error(...args) {
            pink.log(4, args);
        }
    }
}(globalThis))
export default {};
