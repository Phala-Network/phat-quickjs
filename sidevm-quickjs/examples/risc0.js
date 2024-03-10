console.log = Sidevm.inspect;

const INPUT = JSON.stringify({
    "boolean_field": true,
    "critical_data": 47,
    "obj_field": {
        "string_subfield": "hello world",
        "array_subfield": [
            "more",
            "example",
            "text"
        ]
    }
});

async function main() {
    console.log('fetching the guest program');
    // The program compiled from `https://github.com/risc0/risc0/tree/main/examples/json`
    const response = await fetch('https://raw.githubusercontent.com/Phala-Network/phat-quickjs/master/sidevm-quickjs/examples/res/json.elf');
    const program = new Uint8Array(await response.arrayBuffer());
    const stdin = encodeString(INPUT);
    console.log('size of guest program:', program.byteLength);
    console.log('stdin:', stdin);
    const result = Sidevm.experimental.runRisc0Program({
        env: {},
        program,
        args: [],
        stdin,
    });
    console.log('result:', result);
}

function encodeString(s) {
    // Step 1: Convert the string to UTF-8 encoded bytes
    const utf8Bytes = Sidevm.utf8Encode(s);

    // Step 2: Encode the string length
    const stringLength = utf8Bytes.length;
    const lengthBuffer = new ArrayBuffer(4); // a 4-byte buffer to store the length
    const lengthView = new DataView(lengthBuffer);
    lengthView.setUint32(0, stringLength, true); // set the length in little endian

    // Step 3: Calculate padding
    const totalLength = 4 + stringLength; // 4 bytes for the length + string bytes
    const paddingLength = (4 - (totalLength % 4)) % 4; // calculate padding needed to align to 4 bytes
    const padding = new Uint8Array(paddingLength).fill(0); // create the padding

    // Step 4: Concatenate everything into a single Uint8Array
    const encodedString = new Uint8Array(4 + stringLength + paddingLength);
    encodedString.set(new Uint8Array(lengthBuffer), 0); // set the length part
    encodedString.set(utf8Bytes, 4); // set the string's UTF-8 bytes part
    encodedString.set(padding, 4 + utf8Bytes.length); // set the padding part

    return encodedString;
}

main().catch(console.error);
