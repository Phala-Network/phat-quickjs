
(function (g) {
    g.MessageEvent = class MessageEvent extends Event {
        constructor(type, eventInitDict) {
            super(type);
            this.data = eventInitDict.data;
        }
    }
    g.CloseEvent = class CloseEvent extends Event {
        constructor(type, eventInitDict = {}) {
            super(type);
            this.code = eventInitDict.code;
        }
    }

    g.WebSocket = class WebSocket extends EventTarget {
        CONNECTING = 0;
        OPEN = 1;
        CLOSING = 2;
        CLOSED = 3;
        binaryType = 'blob';

        constructor(url, protocols = []) {
            super();
            this.url = url;
            let parsedUrl = new URL(url);
            this.readyState = WebSocket.CONNECTING;
            const headers = new Headers();
            if (protocols && protocols.length > 0) {
                headers.set('Sec-WebSocket-Protocol', protocols.join(','));
            }
            const key = btoa(crypto.getRandomValues(new Uint8Array(16)));
            const options = {
                url,
                headers: {
                    'Host': parsedUrl.host,
                    'Connection': 'Upgrade',
                    'Upgrade': 'websocket',
                    'Sec-WebSocket-Version': '13',
                    'Sec-WebSocket-Key': key,
                },
            };
            this._task = Wapo.wsOpen(options, (cmd, msg) => {
                switch (cmd) {
                    case 'open':
                        this.readyState = WebSocket.OPEN;
                        this._wsTx = msg;
                        this.dispatchEvent(new Event('open'));
                        break;
                    case 'message':
                        switch (msg.kind) {
                            case 'text':
                                msg = msg.data;
                                break;
                            case 'binary':
                                if (this.binaryType === 'arraybuffer') {
                                    msg = new Uint8Array(msg.data).buffer;
                                } else {
                                    msg = new Blob([msg.data]);
                                }
                                break;
                            case 'close':
                                this.readyState = WebSocket.CLOSED;
                                this.dispatchEvent(new CloseEvent('close'));
                                return;
                            case 'ping':
                                Wapo.wsSend(this._wsTx, { kind: 'pong', data: msg.data });
                                return;
                        }
                        this.dispatchEvent(new MessageEvent('message', { data: msg }));
                        break;
                    case 'error':
                        this.readyState = WebSocket.CLOSED;
                        this.dispatchEvent(new CloseEvent('close'));
                        break;
                }
            });
        }

        send(data) {
            let kind = 'text';
            if (data instanceof ArrayBuffer) {
                data = new Uint8Array(data);
                kind = 'binary';
            } else if (typeof data === 'string') {
                kind = 'text';
            } else if (data instanceof Blob) {
                kind = 'binary';
                data = new Uint8Array(data);
            } else if (data instanceof Uint8Array) {
                kind = 'binary';
            } else {
                throw new Error('Unsupported data type');
            }
            Wapo.wsSend(this._wsTx, { kind, data });
        }

        close(code = 1000, reason = '') {
            Wapo.wsClose(this._wsTx, code, reason);
            Wapo.close(this._task);
        }
        get bufferedAmount() {
            return 0;
        }
        set onopen(handler) {
            this.addEventListener('open', handler);
        }
        set onmessage(handler) {
            this.addEventListener('message', handler);
        }
        set onclose(handler) {
            this.addEventListener('close', handler);
        }
        set onerror(handler) {
            this.addEventListener('error', handler);
        }
    }
}(globalThis))