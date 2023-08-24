console.log = Sidevm.inspect;
console.log('Start to listen http requests...');
Sidevm.httpListen((req) => {
    console.log('An http request received:', req);
    Sidevm.httpSendResponse(req.opaqueResponseTx, {
        status: 200,
        headers: {
            'Content-Type': 'text/plain',
            'Content-Length': '0',
        }
    })
});
