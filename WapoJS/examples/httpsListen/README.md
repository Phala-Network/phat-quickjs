## Demo for httpsListen

This example demonstrates how to use the `httpsListen` function. The function enables the guest code to listen to HTTPS requests on a shared port and the incoming requests are dispatched to the guest code based on the SNI name.

## Instructions
```
$ git clone https://github.com/Phala-Network/phat-quickjs --recursive
$ cd phat-quickjs/WapoJS/examples/httpsListen
$ make -C ../../
$ # The option `-u` let the command to remember the engine file, so you can directly run `../../wapojs-run httpsListen.js` next time.
$ ../../wapojs-run -u --engine ../../wapojs.wasm --tls-port 1443 httpsListen.js 
```

This will start a HTTPS server on port 1443. You can access it by visiting `https://localhost:1443` in the browser.

By default, the guest code can listen to arbitrary SNI names. However, the wapod verifies the SNI name and the certificate chain to ensure that certificates are issued by any of the trusted CAs, to prevent DoS attacks.

The enable certificate verification to wapojs-run, you can add `--verfiy-cert` option to the command line. For example:

```
$ ../../wapojs-run -u --engine ../../wapojs.wasm --tls-port 1443 --verify-cert httpsListen.js 
```

