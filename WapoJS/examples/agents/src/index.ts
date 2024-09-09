import "@phala/wapo-env"

import { getConfigOrInit } from "./config";
import { listen } from "./http";
import app from "./app";


async function main() {
    const config = await getConfigOrInit();
    listen(config.tlsConfig, app.fetch);
}

main()
    .catch(err => console.error(err))
