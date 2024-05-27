console.log = Wapo.inspect;
const url = "https://httpbin.kvin.wang:8443/anything";
console.log('Posting data to url:', url);
Wapo.httpRequest({
        url,
        method: "POST",
        bodyText: "0x303132",
        headers: {
            "X-Foo": "Bar",
            "Content-Type": "text/plain",
        }
    },
    (cmd, data) => {
        console.log(`=================[${cmd}]===================`);
        switch (cmd) {
            case "head":
                console.log("head:", data);
                Wapo.streamOpenRead(data.opaqueBodyStream, (cmd, data) => {
                    switch (cmd) {
                        case "data":
                            console.log(`data.length=${data.length}`);
                            console.log('-------------------------------------------');
                            console.log(new TextDecoder().decode(data));
                            break;
                        case "end":
                            break;
                        case "error":
                            console.log("error:", data);
                            break;
                    }
                });
                break;
            case "error":
                console.log("error:", data);
                break;
            default:
                console.log("unknown cmd:", cmd);
                console.log("data:", data);
                break;
        }
    }
);
