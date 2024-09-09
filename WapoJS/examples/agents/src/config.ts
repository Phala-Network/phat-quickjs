export class KeyRing {
    static BOOT_DATA_ENCRYPT_KEY = Wapo.deriveSecret("boot_data_encrypt_key");
    static CACHE_ENCRYPT_KEY = Wapo.deriveSecret("cache_encrypt_key");
}

export async function getConfigOrInit(initFn = defaultConfig) {
    let config: Uint8Array | string = Wapo.bootData();
    if (!config) {
        const lck = Wapo.tryLock("/boot.lck");
        try {
            config = Wapo.bootData();
            if (!config) {
                config = await initFn();
                Wapo.storeBootData(config);
            }
        } finally {
            Wapo.unlock(lck);
        }
    }
    if (typeof config !== "string") {
        config = new TextDecoder().decode(config);
    }
    return JSON.parse(config);
}

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

async function defaultConfig() {
    const CONFIG_SERVER = process.env.CONFIG_SERVER;
    if (!CONFIG_SERVER) {
        return JSON.stringify({
            tlsConfig: tlsConfig,
            maxBodyLen: 1024 * 10,
        });
    }

    // TODO: Get config from a config server
    // we can generate a ecdh key here and sign it by the worker.
    // Then send a config request to the config server with the signature.
    // The server can verify the signature to ensure the request is from this app then return the config.
}
