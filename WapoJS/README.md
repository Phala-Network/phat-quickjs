# WapoJS

A JS Runtime running in Wapod.

## Quick Start

### Build and run a Wapod instance
```bash
git clone https://github.com/Phala-Network/wapo.git --recursive
cd wapo/wapod
cargo run --release -- -m 1g
```

### Build and deploy WapoJS
```bash
git clone https://github.com/Phala-Network/phat-quickjs.git --recursive
cd phat-quickjs/WapoJS
make
WAPOD_URL=http://127.0.0.1:8001 ./examples/deploy.js wapojs.wasm examples/gptProxy.js
```

### Access the deployed WapoJS
Open `http://127.0.0.1:8002/app/<ADDRESS>/` with your browser where `<ADDRESS>` is the address
displayed in the previous step.

**NOTE: The HTML page is for development only. Don't use Wapod to provide HTML pages in production.**
