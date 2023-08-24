console.log = Sidevm.inspect;
console.log('Start to listen http requests...');
Sidevm.httpListen((req) => {
    console.log('An http request received:', req);
});
