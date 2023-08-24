console.log = Sidevm.inspect;
Sidevm.httpListen((req) => {
    console.log('receive http request');
    console.log(req)
});
setTimeout(() => {
    console.log('httpListen')
}, 1000);
