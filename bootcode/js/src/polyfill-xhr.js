(function (g) {
    var InvalidStateError, NetworkError, ProgressEvent, SecurityError, SyntaxError, XMLHttpRequest, XMLHttpRequestEventTarget, XMLHttpRequestUpload;
    XMLHttpRequestEventTarget = (function () {
        class XMLHttpRequestEventTarget {
            constructor() {
                this.onloadstart = null;
                this.onprogress = null;
                this.onabort = null;
                this.onerror = null;
                this.onload = null;
                this.ontimeout = null;
                this.onloadend = null;
                this._listeners = {};
            }
            addEventListener(eventType, listener) {
                var base;
                eventType = eventType.toLowerCase();
                (base = this._listeners)[eventType] || (base[eventType] = []);
                this._listeners[eventType].push(listener);
                return void 0;
            }
            removeEventListener(eventType, listener) {
                var index;
                eventType = eventType.toLowerCase();
                if (this._listeners[eventType]) {
                    index = this._listeners[eventType].indexOf(listener);
                    if (index !== -1) {
                        this._listeners[eventType].splice(index, 1);
                    }
                }
                return void 0;
            }
            dispatchEvent(event) {
                var eventType, j, len, listener, listeners;
                event.currentTarget = event.target = this;
                eventType = event.type;
                if (listeners = this._listeners[eventType]) {
                    for (j = 0, len = listeners.length; j < len; j++) {
                        listener = listeners[j];
                        listener.call(this, event);
                    }
                }
                if (listener = this[`on${eventType}`]) {
                    listener.call(this, event);
                }
                return void 0;
            }

        };

        XMLHttpRequestEventTarget.prototype.onloadstart = null;
        XMLHttpRequestEventTarget.prototype.onprogress = null;
        XMLHttpRequestEventTarget.prototype.onabort = null;
        XMLHttpRequestEventTarget.prototype.onerror = null;
        XMLHttpRequestEventTarget.prototype.onload = null;
        XMLHttpRequestEventTarget.prototype.ontimeout = null;
        XMLHttpRequestEventTarget.prototype.onloadend = null;
        return XMLHttpRequestEventTarget;

    }).call(this);

    XMLHttpRequest = (function () {
        class XMLHttpRequest extends XMLHttpRequestEventTarget {
            constructor(options) {
                super();
                this.onreadystatechange = null;
                this._anonymous = options && options.anon;
                this.readyState = XMLHttpRequest.UNSENT;
                this.response = null;
                this.responseText = '';
                this.responseType = '';
                this.responseURL = '';
                this.status = 0;
                this.statusText = '';
                this.timeout = 0;
                this.upload = new XMLHttpRequestUpload(this);
                this._method = null; // String
                this._sync = false;
                this._headers = null; // Object<String, String>
                this._loweredHeaders = null; // Object<lowercase String, String>
                this._mimeOverride = null;
                this._request = null; // http.ClientRequest
                this._response = null; // http.ClientResponse
                this._responseParts = null; // Array<Buffer, String>
                this._responseHeaders = null; // Object<lowercase String, String>
                this._aborting = null;
                this._error = null;
                this._loadedBytes = 0;
                this._totalBytes = 0;
                this._lengthComputable = false;
            }

            open(method, url, async, user, password) {
                method = method.toUpperCase();
                if (method in this._restrictedMethods) {
                    throw new SecurityError(`HTTP method ${method} is not allowed in XHR`);
                }
                if (async === void 0) {
                    async = true;
                }
                switch (this.readyState) {
                    case XMLHttpRequest.UNSENT:
                    case XMLHttpRequest.OPENED:
                    case XMLHttpRequest.DONE:
                        // Nothing to do here.
                        null;
                        break;
                    case XMLHttpRequest.HEADERS_RECEIVED:
                    case XMLHttpRequest.LOADING:
                        // TODO(pwnall): terminate abort(), terminate send()
                        null;
                }
                this._method = method;
                this.url = url;
                this._sync = !async;
                this._headers = {};
                this._loweredHeaders = {};
                this._mimeOverride = null;
                this._setReadyState(XMLHttpRequest.OPENED);
                this._request = null;
                this._response = null;
                this.status = 0;
                this.statusText = '';
                this._responseParts = [];
                this._responseHeaders = null;
                this._loadedBytes = 0;
                this._totalBytes = 0;
                this._lengthComputable = false;
                return void 0;
            }

            setRequestHeader(name, value) {
                var loweredName;
                if (this.readyState !== XMLHttpRequest.OPENED) {
                    throw new InvalidStateError("XHR readyState must be OPENED");
                }
                loweredName = name.toLowerCase();
                if (this._restrictedHeaders[loweredName] || /^sec\-/.test(loweredName) || /^proxy-/.test(loweredName)) {
                    console.warn(`Refused to set unsafe header \"${name}\"`);
                    return void 0;
                }
                value = value.toString();
                if (loweredName in this._loweredHeaders) {
                    // Combine value with the existing header value.
                    name = this._loweredHeaders[loweredName];
                    this._headers[name] = this._headers[name] + ', ' + value;
                } else {
                    // New header.
                    this._loweredHeaders[loweredName] = name;
                    this._headers[name] = value;
                }
                return void 0;
            }

            send(data) {
                if (this.readyState !== XMLHttpRequest.OPENED) {
                    throw new InvalidStateError("XHR readyState must be OPENED");
                }
                if (this._request) {
                    throw new InvalidStateError("send() already called");
                }
                this._sendHttp(data);
                return void 0;
            }

            abort() {
                if (!this._request) {
                    return;
                }
                this._request.abort();
                this._setError();
                this._dispatchProgress('abort');
                this._dispatchProgress('loadend');
                return void 0;
            }

            getResponseHeader(name) {
                var loweredName;
                if (!this._responseHeaders) {
                    return null;
                }
                loweredName = name.toLowerCase();
                if (loweredName in this._responseHeaders) {
                    return this._responseHeaders[loweredName];
                } else {
                    return null;
                }
            }

            getAllResponseHeaders() {
                var lines, name, value;
                if (!this._responseHeaders) {
                    return '';
                }
                lines = (function () {
                    var ref, results;
                    ref = this._responseHeaders;
                    results = [];
                    for (name in ref) {
                        value = ref[name];
                        results.push(`${name}: ${value}`);
                    }
                    return results;
                }).call(this);
                return lines.join("\r\n");
            }

            overrideMimeType(newMimeType) {
                if (this.readyState === XMLHttpRequest.LOADING || this.readyState === XMLHttpRequest.DONE) {
                    throw new InvalidStateError("overrideMimeType() not allowed in LOADING or DONE");
                }
                this._mimeOverride = newMimeType.toLowerCase();
                return void 0;
            }

            _setReadyState(newReadyState) {
                var event;
                this.readyState = newReadyState;
                event = new ProgressEvent('readystatechange');
                this.dispatchEvent(event);
                return void 0;
            }
            _sendHttp(data) {
                if (this._sync) {
                    throw new Error("Synchronous XHR processing not implemented");
                }
                if ((data != null) && (this._method === 'GET' || this._method === 'HEAD')) {
                    console.warn(`Discarding entity body for ${this._method} requests`);
                    data = null;
                } else {
                    // Send Content-Length: 0
                    data || (data = '');
                }
                // NOTE: this is called before finalizeHeaders so that the uploader can
                //       figure out Content-Length and Content-Type.
                this.upload._setData(data);
                this._finalizeHeaders();
                this._sendHxxpRequest();
                return void 0;
            }
            _sendHxxpRequest() {
                var agent = "sidejs/1.0.0";
                const request = {};
                const response = {
                    headers: {},
                    body: [],
                };
                const reqId = Sidevm.httpRequest({
                    url: this.url,
                    method: this._method,
                    headers: {
                        "User-Agent": agent,
                        ...this._headers,
                    },
                    timeout: this.timeout || 60000,
                    body: this.upload._body,
                    callback: (cmd, data) => {
                        switch (cmd) {
                            case "head":
                                {
                                    const head = data;
                                    response.ok = ((head.status / 100) | 0) == 2;
                                    response.headers = data.headers;
                                    response.statusText = head.statusText;
                                    response.statusCode = head.status;
                                    response.url = head["Location"] || head["location"] || "";
                                    response.bodyUsed = false;
                                    response.type = "default";
                                    this._onHttpResponse(request, response);
                                    break;
                                }
                            case "data":
                                this._onHttpResponseData(response, data);
                                break;
                            case "end": {
                                this._onHttpResponseEnd(response);
                                this._onHttpResponseClose(response);
                                break;
                            }
                            case "error": {
                                this._onHttpRequestError(request, data);
                                break;
                            }
                        }
                    }
                });
                request.abort = () => {
                    Sidevm.close(reqId);
                };
                this._request = request;
                this._dispatchProgress('loadstart');
                return void 0;
            }

            _finalizeHeaders() {
                var base;
                this._headers['Connection'] = 'keep-alive';
                if (this._anonymous) {
                    this._headers['Referer'] = 'about:blank';
                }
                (base = this._headers)['User-Agent'] || (base['User-Agent'] = this._userAgent);
                this.upload._finalizeHeaders(this._headers, this._loweredHeaders);
                return void 0;
            }

            _onHttpResponse(request, response) {
                var lengthString;
                if (this._request !== request) {
                    return;
                }
                // Transparent redirection handling.
                switch (response.statusCode) {
                    case 301:
                    case 302:
                    case 303:
                    case 307:
                    case 308:
                        this._method = 'GET';
                        if ('content-type' in this._loweredHeaders) {
                            delete this._headers[this._loweredHeaders['content-type']];
                            delete this._loweredHeaders['content-type'];
                        }
                        // XMLHttpRequestUpload#_finalizeHeaders() sets Content-Type directly.
                        if ('Content-Type' in this._headers) {
                            delete this._headers['Content-Type'];
                        }
                        // Restricted headers can't be set by the user, no need to check
                        // loweredHeaders.
                        delete this._headers['Content-Length'];
                        this.upload._reset();
                        this._finalizeHeaders();
                        this._sendHxxpRequest();
                        return;
                }
                this._response = response;
                this.status = this._response.statusCode;
                this.statusText = this._response.statusText;
                this._parseResponseHeaders(response);
                if (lengthString = this._responseHeaders['content-length']) {
                    this._totalBytes = parseInt(lengthString);
                    this._lengthComputable = true;
                } else {
                    this._lengthComputable = false;
                }
                return this._setReadyState(XMLHttpRequest.HEADERS_RECEIVED);
            }

            _onHttpResponseData(response, data) {
                if (this._response !== response) {
                    return;
                }
                this._responseParts.push(data);
                this._loadedBytes += data.length;
                if (this.readyState !== XMLHttpRequest.LOADING) {
                    this._setReadyState(XMLHttpRequest.LOADING);
                }
                return this._dispatchProgress('progress');
            }

            _onHttpResponseEnd(response) {
                if (this._response !== response) {
                    return;
                }
                this._parseResponse();
                this._request = null;
                this._response = null;
                this._setReadyState(XMLHttpRequest.DONE);
                this._dispatchProgress('load');
                return this._dispatchProgress('loadend');
            }

            _onHttpResponseClose(response) {
                var request;
                if (this._response !== response) {
                    return;
                }
                request = this._request;
                this._setError();
                request.abort();
                this._setReadyState(XMLHttpRequest.DONE);
                this._dispatchProgress('error');
                return this._dispatchProgress('loadend');
            }

            _onHttpTimeout(request) {
                if (this._request !== request) {
                    return;
                }
                this._setError();
                request.abort();
                this._setReadyState(XMLHttpRequest.DONE);
                this._dispatchProgress('timeout');
                return this._dispatchProgress('loadend');
            }

            _onHttpRequestError(request, error) {
                if (this._request !== request) {
                    return;
                }
                this._setError(error);
                request.abort();
                this._setReadyState(XMLHttpRequest.DONE);
                this._dispatchProgress('error');
                return this._dispatchProgress('loadend');
            }

            _dispatchProgress(eventType) {
                var event;
                event = new ProgressEvent(eventType);
                event.lengthComputable = this._lengthComputable;
                event.loaded = this._loadedBytes;
                event.total = this._totalBytes;
                this.dispatchEvent(event);
                return void 0;
            }

            _setError(error) {
                this._request = null;
                this._response = null;
                this._responseHeaders = null;
                this._responseParts = null;
                if (error) {
                    this.responseText = error;
                }
                return void 0;
            }

            _parseResponseHeaders(response) {
                var loweredName, name, ref, value;
                this._responseHeaders = {};
                ref = response.headers;
                for (name in ref) {
                    value = ref[name];
                    loweredName = name.toLowerCase();
                    if (this._privateHeaders[loweredName]) {
                        continue;
                    }
                    if (this._mimeOverride !== null && loweredName === 'content-type') {
                        value = this._mimeOverride;
                    }
                    this._responseHeaders[loweredName] = value;
                }
                if (this._mimeOverride !== null && !('content-type' in this._responseHeaders)) {
                    this._responseHeaders['content-type'] = this._mimeOverride;
                }
                return void 0;
            }

            _parseResponse() {
                var arrayBuffer, buffer, i, j, jsonError, ref, view;
                buffer = Sidevm.concatU8a(this._responseParts);
                this._responseParts = null;
                switch (this.responseType) {
                    case 'text':
                        this._parseTextResponse(buffer);
                        break;
                    case 'json':
                        this.responseText = null;
                        try {
                            this.response = JSON.parse(buffer.toString('utf-8'));
                        } catch (error1) {
                            jsonError = error1;
                            this.response = null;
                        }
                        break;
                    case 'buffer':
                        this.responseText = null;
                        this.response = buffer;
                        break;
                    case 'arraybuffer':
                        this.responseText = null;
                        arrayBuffer = new ArrayBuffer(buffer.length);
                        view = new Uint8Array(arrayBuffer);
                        for (i = j = 0, ref = buffer.length; (0 <= ref ? j < ref : j > ref); i = 0 <= ref ? ++j : --j) {
                            view[i] = buffer[i];
                        }
                        this.response = arrayBuffer;
                        break;
                    default:
                        // TODO(pwnall): content-base detection
                        this._parseTextResponse(buffer);
                }
                return void 0;
            }

            _parseTextResponse(buffer) {
                try {
                    this.responseText = new TextDecoder().decode(buffer);
                } catch (error1) {
                    console.error('Failed to parse text response', error1);
                }
                this.response = this.responseText;
                return void 0;
            }

            _parseResponseEncoding() {
                var contentType, encoding, match;
                encoding = null;
                if (contentType = this._responseHeaders['content-type']) {
                    if (match = /\;\s*charset\=(.*)$/.exec(contentType)) {
                        return match[1];
                    }
                }
                return 'utf-8';
            }
        };

        XMLHttpRequest.prototype.onreadystatechange = null;
        XMLHttpRequest.prototype.readyState = null;
        XMLHttpRequest.prototype.response = null;
        XMLHttpRequest.prototype.responseText = null;
        XMLHttpRequest.prototype.responseType = null;
        XMLHttpRequest.prototype.status = null;
        XMLHttpRequest.prototype.timeout = null;
        XMLHttpRequest.prototype.upload = null;
        XMLHttpRequest.prototype.UNSENT = 0;
        XMLHttpRequest.UNSENT = 0;
        XMLHttpRequest.prototype.OPENED = 1;
        XMLHttpRequest.OPENED = 1;
        XMLHttpRequest.prototype.HEADERS_RECEIVED = 2;
        XMLHttpRequest.HEADERS_RECEIVED = 2;
        XMLHttpRequest.prototype.LOADING = 3;
        XMLHttpRequest.LOADING = 3;
        XMLHttpRequest.prototype.DONE = 4;
        XMLHttpRequest.DONE = 4;
        XMLHttpRequest.prototype._restrictedMethods = {
            CONNECT: true,
            TRACE: true,
            TRACK: true
        };
        XMLHttpRequest.prototype._restrictedHeaders = {
            'accept-charset': true,
            'accept-encoding': true,
            'access-control-request-headers': true,
            'access-control-request-method': true,
            connection: true,
            'content-length': true,
            cookie: true,
            cookie2: true,
            date: true,
            dnt: true,
            expect: true,
            host: true,
            'keep-alive': true,
            origin: true,
            referer: true,
            te: true,
            trailer: true,
            'transfer-encoding': true,
            upgrade: true,
            via: true
        };
        XMLHttpRequest.prototype._privateHeaders = {
            'set-cookie': true,
            'set-cookie2': true
        };
        XMLHttpRequest.prototype._userAgent = `Sidevm HTTP Client`;

        return XMLHttpRequest;

    }).call(this);

    module.exports = XMLHttpRequest;
    XMLHttpRequest.XMLHttpRequest = XMLHttpRequest;
    SecurityError = class SecurityError extends Error { };
    XMLHttpRequest.SecurityError = SecurityError;
    InvalidStateError = class InvalidStateError extends Error { };
    InvalidStateError = class InvalidStateError extends Error { };
    XMLHttpRequest.InvalidStateError = InvalidStateError;
    NetworkError = class NetworkError extends Error { };
    XMLHttpRequest.SyntaxError = SyntaxError;
    SyntaxError = class SyntaxError extends Error { };

    ProgressEvent = (function () {
        class ProgressEvent {
            constructor(type) {
                this.type = type;
                this.target = null;
                this.currentTarget = null;
                this.lengthComputable = false;
                this.loaded = 0;
                this.total = 0;
            }

        };

        ProgressEvent.prototype.bubbles = false;
        ProgressEvent.prototype.cancelable = false;
        ProgressEvent.prototype.target = null;
        ProgressEvent.prototype.loaded = null;
        ProgressEvent.prototype.lengthComputable = null;
        ProgressEvent.prototype.total = null;

        return ProgressEvent;

    }).call(this);

    XMLHttpRequest.ProgressEvent = ProgressEvent;
    XMLHttpRequestUpload = class XMLHttpRequestUpload extends XMLHttpRequestEventTarget {
        constructor(request) {
            super();
            this._request = request;
            this._reset();
        }
        _reset() {
            this._contentType = null;
            this._body = null;
            return void 0;
        }
        _setData(data) {
            if (typeof data === 'undefined' || data === null) {
                return;
            }
            if (typeof data === 'string') {
                if (data.length !== 0) {
                    this._contentType = 'text/plain;charset=UTF-8';
                }
                this._body = new TextEncoder().encode(data);
            } else if (data instanceof ArrayBuffer) {
                this._body = data;
            }
            return void 0;
        }
        _finalizeHeaders(headers, loweredHeaders) {
            if (this._contentType) {
                if (!('content-type' in loweredHeaders)) {
                    headers['Content-Type'] = this._contentType;
                }
            }
            if (this._body) {
                headers['Content-Length'] = this._body.length.toString();
            }
            return void 0;
        }
    };

    XMLHttpRequest.XMLHttpRequestUpload = XMLHttpRequestUpload;
    g.XMLHttpRequest = XMLHttpRequest;
}(globalThis));