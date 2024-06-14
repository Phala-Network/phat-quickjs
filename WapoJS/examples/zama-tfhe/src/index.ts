import "./patchWasm"
import {
    CompactFheUint8List,
    TfheCompactPublicKey,
    TfheConfigBuilder,
    TfheClientKey,
    ShortintParameters,
    ShortintParametersName,
} from "node-tfhe";

function createTfheKeypair() {
    const block_params = new ShortintParameters(
      ShortintParametersName.PARAM_MESSAGE_2_CARRY_2_COMPACT_PK_PBS_KS,
    );
    const config = TfheConfigBuilder.default()
      .use_custom_parameters(block_params)
      .build();
    const clientKey = TfheClientKey.generate(config);
    let publicKey = TfheCompactPublicKey.new(clientKey);
    publicKey = TfheCompactPublicKey.deserialize(publicKey.serialize());
    return { clientKey, publicKey };
};

function encryptUint8(value: number, pubkey: TfheCompactPublicKey): Uint8Array {
    const uint8Array = new Uint8Array([value]);
    const encrypted = CompactFheUint8List.encrypt_with_compact_public_key(
      uint8Array,
      pubkey,
    );
    return encrypted.serialize();
};

function main() {
    const { clientKey, publicKey } = createTfheKeypair();
    const value = 100;
    const encrypted = encryptUint8(value, publicKey);
    console.log(`Encrypted: ${encrypted}`);
    const compactList = CompactFheUint8List.deserialize(encrypted);
    let encryptedList = compactList.expand();
    encryptedList.forEach((v) => {
      const decrypted = v.decrypt(clientKey);
      console.log(`Decrypted: ${decrypted}`);
    });
}

try {
  main()
} finally {
  process.exit(0)
}
