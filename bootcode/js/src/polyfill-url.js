(function (g) {
    g.URL = class URL {
        constructor(url, base) {
            const pairs = sidevm.parseURL(url, base);
            this._pairs = pairs;
            this.hash = pairs['hash'];
            this.host = pairs['host'];
            this.hostname = pairs['hostname'];
            this.origin = pairs['origin'];
            this.password = pairs['password'];
            this.pathname = pairs['pathname'];
            this.port = pairs['port'];
            this.protocol = pairs['protocol'];
            this.search = pairs['search'];
            this.username = pairs['username'];
        }
        get href() {
            return this.toString();
        }
        searchParams() {
            return new URLSearchParams(this.search);
        }
        toString() {
            let result = this.protocol + '//';
            if (this.username) {
                result += this.username;
                if (this.password) {
                    result += ':' + this.password;
                }
                result += '@';
            }
            result += this.hostname;
            if (this.port) {
                result += ':' + this.port;
            }
            result += this.pathname;
            if (this.search) {
                result += this.search;
            }
            if (this.hash) {
                result += this.hash;
            }
            return result;
        }

    }
    g.URLSearchParams = class URLSearchParams {
        constructor(options) {
            this.params = new Map();
            if (typeof options == 'string') {
                // __hostCall returns an array of [key, value] pairs
                options = sidevm.parseURLParams(options);
            }
            for (const [key, value] of options) {
                this.params.set(key, value);
            }
        }

        keys() {
            return this.params.keys();
        }

        has(n) {
            return this.params.has(n);
        }

        values() {
            return this.params.values();
        }

        entries() {
            return this.params.entries();
        }

        append(key, value) {
            this.params.set(key, value);
        }

        delete(key) {
            this.params.delete(key);
        }

        get(key) {
            return this.params.get(key);
        }

        getAll(key) {
            return Array.from(this.params.entries()).filter(([k]) => k === key).map(([, v]) => v);
        }

        set(key, value) {
            this.params.set(key, value);
        }

        forEach(callback) {
            this.params.forEach(callback);
        }

        toString() {
            let result = '';
            for (const [key, value] of this.params) {
                result += `${encodeURIComponent(key)}=${encodeURIComponent(value)}&`;
            }
            return result.slice(0, -1); // Remove the trailing '&'
        }

        toJSON() {
            return this.toString();
        }
    }
})(globalThis);