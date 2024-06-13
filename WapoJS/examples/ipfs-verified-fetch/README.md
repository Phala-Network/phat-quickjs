# Build & Run
```
$ git clone https://github.com/Phala-Network/phat-quickjs --recursive
$ cd phat-quickjs/WapoJS/examples/ipfs-verified-fetch
$ yarn && yarn build
$ make -C ../../ native
$ ../../wapojs dist/index.js
```

With run-js:

```
# at WapoJS/examples/ipfs-verified-fetch
yarn build
# at WapoJS (make sure 8002 is not occupied)
./run-js.sh examples/ipfs-verified-fetch/dist/index.js
# access via browser
http://localhost:8002/app/0x0000000000000000000000000000000000000000000000000000000000000000/
```
