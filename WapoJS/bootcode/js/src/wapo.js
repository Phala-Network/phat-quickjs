(function (g) {
    const objectInspect = require("object-inspect");
    const { formatError } = require("pretty-print-error/dist/index.js");

    function inspect(obj) {
        if(
          obj instanceof Error ||
          (obj !== null && typeof obj === 'object' && 'message' in obj && 'stack' in obj)
        ) {
            return formatError(obj);
        } else {
            return objectInspect(obj);
        }
    }

    function timerFn(hostFn) {
        return function (f, t) {
            t = Math.round(t || 0);
            if (typeof f == 'string') {
                return hostFn(() => eval(f), t);
            }
            const args = Array.prototype.slice.call(arguments, 2);
            const callback = () => { f.apply(null, args); };
            return hostFn(callback, t);
        }
    }
    function concatU8a(arrays) {
        const totalSize = arrays.reduce((acc, e) => acc + e.length, 0);
        const merged = new Uint8Array(totalSize);
        arrays.forEach((array, i, arrays) => {
            const offset = arrays.slice(0, i).reduce((acc, e) => acc + e.length, 0);
            merged.set(array, offset);
        });
        return merged;
    }
    g.Wapo.concatU8a = concatU8a;
    g.setTimeout = timerFn(Wapo.setTimeout);
    g.setInterval = timerFn(Wapo.setInterval);
    g.clearTimeout = function (id) {
        if (id == null) return;
        return Wapo.close(id);
    };
    g.clearInterval = g.clearTimeout;
    g.setImmediate = (f, ...args) => setTimeout(f, 0, ...args);

    g.Wapo.inspect = inspect;

    const directConsole = {
        log(...args) {
            return Wapo.print(2, args.map(inspect));
        },
        info(...args) {
            return Wapo.print(2, args.map(inspect));
        },
        warn(...args) {
            return Wapo.print(3, args.map(inspect));
        },
        error(...args) {
            return Wapo.print(4, args.map(inspect));
        }
    }

    let bufferedLogs = [];
    function appendLogRecord(level, args) {
        const obj = {
            message: args.map(inspect).join(" "),
            time: new Date().getTime(),
            level: level,
        }
        bufferedLogs.push(obj);
    }
    const bufferedConsole = {
        log(...args) {
            appendLogRecord(2, args);
        },
        info(...args) {
            appendLogRecord(2, args);
        },
        warn(...args) {
            appendLogRecord(3, args);
        },
        error(...args) {
            appendLogRecord(4, args);
        }
    }

    Object.defineProperty(g, "scriptLogs", {
        get() {
            return JSON.stringify(bufferedLogs);
        },
        writeable: false,
    });

    g.console = directConsole;
    g.Wapo.useBufferedLogs = function (flag) {
        if (flag) {
            g.console = bufferedConsole;
        } else {
            g.console = directConsole;
        }
    }

    g.print = directConsole.log;

    g.btoa = s => Wapo.base64Encode(s, true);
    g.atob = d => Wapo.base64Decode(d, true);
    g.global = g;
    g.window = g;
    g.SCALE = Wapo.SCALE;
    g.process = {
        env: Wapo.env,
        exit: Wapo.exit,
        get argv() {
            return ["wapojs", "<eval>", ...scriptArgs];
        },
        version: "v0.9.0",
        nextTick: setImmediate,
        stdout: {
            write: function (s) {
                return Wapo.print(2, s);
            },
            fd: 1,
        },
        stderr: {
            write: function (s) {
                return Wapo.print(4, s);
            },
            fd: 2,
        },
        cwd: () => "/",
    };
    g.performance = {
        now() {
            return new Date().getTime();
        }
    };
    g.localStorage = {
        _data: {},
        getItem(key) {
            return this._data[key];
        },
        setItem(key, value) {
            this._data[key] = value;
        },
        removeItem(key) {
            delete this._data[key];
        }
    };
    Error.captureStackTrace = function (error) {
        class DummyCallSite {
            constructor() {
            }
            getFileName() {
                return "<eval>";
            }
            isEval() {
                return true;
            }
            getFunctionName() {
                return "<unknown>";
            }
            getFunction() {}
            getColumnNumber() {
                return 0;
            }
            getLineNumber() {
                return 0;
            }
            getEvalOrigin() {
                return "<unknown>";
            }
            isNative() {
                return false;
            }
        }
        error.stack = [new DummyCallSite(), new DummyCallSite(), new DummyCallSite()]
    };


    const _version = g.Wapo.version;
    Object.defineProperty(g.Wapo, "version", {
        get() {
            return _version;
        },
        writeable: false,
        configurable: false,
    });

    const _workerSecret = g.Wapo.workerSecret;
    Object.defineProperty(g.Wapo, "workerSecret", {
        get() {
            return _workerSecret;
        },
        writeable: false,
        configurable: false,
    });

    Object.defineProperty(g.Wapo, "deriveSecret", {
        value: function (message) {
            return Wapo.hash("blake2b512", `${_workerSecret}${message}`);
        },
        writeable: false,
        configurable: false,
    });

    // should be called in host mode only.
    g.Wapo.run = async function (code, options) {
        const defaultOptions = {
            args: [],
            env: {},
            timeLimit: 120_000, // 2 minutes
            gasLimit: 100_000,
            memoryLimit: 1024 * 1024 * 128, // 128 MB
            polyfills: ['nodejs'],
        };
        options = { ...defaultOptions, ...(options || {}) };
        const result = await new Promise((resolve) => Wapo.isolateEval({
            scripts: ["Wapo.useBufferedLogs(true);", code],
            args: options.args,
            env: options.env,
            timeLimit: options.timeLimit,
            gasLimit: options.gasLimit,
            memoryLimit: options.memoryLimit,
            polyfills: options.polyfills,
        }, resolve)).then(([error, value, serialized, logs]) => {
            if (serialized) {
                try {
                    value = JSON.parse(serialized)?.output;
                } catch (e) {
                }
            }
            try {
                logs = JSON.parse(logs);
            } catch (e) {
                logs = [];
            }
            const result = {
                error,
                value,
                logs,
                isError: !!error,
                isOk: !error,
            };
            Object.freeze(result);
            return result;
        });
        return result;
    }

    // should be called in guest mode only.
    g.Wapo.callModuleEntry = async function callModuleEntry() {
        const fn = globalThis.module?.exports;
        if (typeof fn === 'function') {
            try {
                const thenable = fn();
                if (thenable && typeof thenable.then === 'function') {
                    const output = await thenable;
                    if (output !== undefined) {
                        globalThis.scriptOutput = output;
                    }
                } else if (thenable !== undefined) {
                    globalThis.scriptOutput = thenable;
                }
            } catch (e) {
                appendLogRecord(4, [e]);
            }
            if (globalThis.scriptOutput !== undefined) {
                try {
                    globalThis.serializedScriptOutput = JSON.stringify({output: globalThis.scriptOutput});
                } catch (e) {
                }
            }
        }
    }
}(globalThis))

export default {};
