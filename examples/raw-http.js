console.log = Sidevm.inspect;
Sidevm.httpRequest({
        url: "https://httpbin.kvin.wang:8443/anything",
        method: "POST",
        bodyText: "0x303132",
    },
    (cmd, data) => {
        console.log(`=================[${cmd}]===================`);
        switch (cmd) {
            case "head":
                console.log("head:", data);
                break;
            case "data":
                console.log(`data.length=${data.length}`);
                console.log("dataText", new TextDecoder().decode(data));
                break;
            case "end":
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
