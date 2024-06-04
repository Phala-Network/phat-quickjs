import { Keyring } from '@polkadot/api';
import { waitReady } from '@polkadot/wasm-crypto';

async function main() {
    await waitReady();
    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
    // Log some info
    console.log(`${alice.meta.name}: has address ${alice.address} with publicKey [${alice.publicKey}]`);
}

main().catch(console.error).finally(() => process.exit());
