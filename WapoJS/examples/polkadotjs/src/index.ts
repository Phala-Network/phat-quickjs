import { Keyring } from '@polkadot/api';
import { waitReady } from '@polkadot/wasm-crypto';
import { stringToU8a, u8aToHex } from '@polkadot/util';

async function main() {
    await waitReady();
    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
    // Log some info
    console.log(`${alice.meta.name}: has address ${alice.address} with publicKey [${alice.publicKey}]`);
    // Convert message, sign and then verify
    const message = stringToU8a('this is our message');
    const signature = alice.sign(message);
    const isValid = alice.verify(message, signature, alice.publicKey);
    // Log info
    console.log(`The signature ${u8aToHex(signature)}, is ${isValid ? '' : 'in'}valid`);
}

main().catch(console.error).finally(() => process.exit());
