# Description

This example demonstrates how to create a simple js execution agent that can be used to execute js code from IPFS.


# Development

To run the example, execute the following command:

```bash
git clone --recursive https://github.com/Phala-Network/phat-quickjs

# install the wapojs x86_64 version for develepment usage
cd phat-quickjs/WapoJS
make install

# checkout & build this example
git checkout agents
cd examples/agents
yarn && yarn build

# run the example
wapojs --tls-port 8443 dist/index.js
```

Then open the browser and navigate to `https://localhost:8443/` to see the result.