console.log = Sidevm.inspect;
console.log('Start to listen http requests...');
Sidevm.httpListen((req) => {
    console.log('Incomming HTTP request:', req);
    Sidevm.httpSendResponse(req.opaqueResponseTx, {
        status: 200,
        headers: {
            'Content-Type': 'text/plain',
            'Content-Length': '0',
            'X-Foo': 'Bar',
        }
    })
});
