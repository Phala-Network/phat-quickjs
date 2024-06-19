console.log = Wapo.inspect;
console.log('Start to listen http requests...');

const CERT = `-----BEGIN CERTIFICATE-----
MIIBZzCCAQ2gAwIBAgIIbELHFTzkfHAwCgYIKoZIzj0EAwIwITEfMB0GA1UEAwwW
cmNnZW4gc2VsZiBzaWduZWQgY2VydDAgFw03NTAxMDEwMDAwMDBaGA80MDk2MDEw
MTAwMDAwMFowITEfMB0GA1UEAwwWcmNnZW4gc2VsZiBzaWduZWQgY2VydDBZMBMG
ByqGSM49AgEGCCqGSM49AwEHA0IABOoRzdEagFDZf/im79Z5JUyeXP96Yww6nH8X
ROvXOESnE0yFtlVjdj0NTNXT2m+PWzuxsjvPVBWR/tpDldjTW8CjLTArMCkGA1Ud
EQQiMCCCE2hlbGxvLndvcmxkLmV4YW1wbGWCCWxvY2FsaG9zdDAKBggqhkjOPQQD
AgNIADBFAiEAsuZKsdksPsrnJFdV9JTZ1P782IlqjqNL9aAURvrF3UkCIDDpTvE5
EyZ5zRflnB+ZwomjXNhTAnasRjQTDqXFrQbP
-----END CERTIFICATE-----`;

const KEY = `-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgH1VlVX/3DI37UR5g
tGzUOSAaOmjQbZMJQ2Z9eBnzh3+hRANCAATqEc3RGoBQ2X/4pu/WeSVMnlz/emMM
Opx/F0Tr1zhEpxNMhbZVY3Y9DUzV09pvj1s7sbI7z1QVkf7aQ5XY01vA
-----END PRIVATE KEY-----`;

const tlsConfig = {
    serverName: "localhost",
    certificateChain: CERT,
    privateKey: KEY,
}

Wapo.httpsListen(tlsConfig, async req => {
    console.log('Incomming HTTP request:', req);
    var body = '';
    if (req.method === "POST") {
        body = await receiveBody(req.opaqueInputStream);
    }

    console.log('Received body of length:', body.length);
    Wapo.httpsSendResponseHead(req.opaqueResponseTx, {
        status: 200,
        headers: {
            'Content-Type': 'text/plain',
            'X-Foo': 'Bar',
        }
    });
    const writer = Wapo.streamOpenWrite(req.opaqueOutputStream);
    await writeString(writer, `You have sent me the following info\n`);
    await writeString(writer, `method: ${req.method}\n`);
    await sleep(1000);
    await writeString(writer, `url: ${req.url}\n`);
    await sleep(1000);
    await writeString(writer, `headers: \n`);
    for (var p of req.headers) {
        await writeString(writer, `    ${p[0]}: ${p[1]}\n`);
        await sleep(500);
    }
    await writeString(writer, `actual body length: ${body.length}\n`);
    console.log('Response sent, closing writer');
    Wapo.streamClose(writer);
});

async function receiveBody(streamHandle) {
    return new Promise((resolve, reject) => {
        const chunks = [];
        Wapo.streamOpenRead(streamHandle, (cmd, data) => {
            switch (cmd) {
                case "data":
                    chunks.push(data);
                    console.log(`Received a chunk, length=${data.length}`);
                    break;
                case "error":
                    reject(data);
                    break;
                case "end":
                    resolve(Wapo.concatU8a(chunks));
                    break;
                default:
                    console.log("unknown cmd:", cmd);
                    break;
            }
        });
    });
}

async function writeString(writer, s) {
    const data = new TextEncoder().encode(s);
    return new Promise((resolve, reject) => {
        Wapo.streamWriteChunk(writer, data, (suc, err) => {
            if (suc) {
                resolve();
            } else {
                reject(err);
            }
        });
    });
}

async function sleep(ms) {
    return new Promise((resolve) => {
        setTimeout(resolve, ms);
    });
}
