async function encrypt(message, secretKey, iv) {
    return await window.crypto.subtle.encrypt(
        {
            name: "AES-GCM",
            iv: iv,
        },
        secretKey,
        message
    );
}

async function decrypt(ciphertext, secretKey, iv) {
    return await window.crypto.subtle.decrypt(
        {
            name: "AES-GCM",
            iv: iv,
        },
        secretKey,
        ciphertext
    );
}

function deriveSecretKey(privateKey, publicKey) {
    return window.crypto.subtle.deriveKey(
        {
            name: "ECDH",
            public: publicKey,
        },
        privateKey,
        {
            name: "AES-GCM",
            length: 256,
        },
        false,
        ["encrypt", "decrypt"]
    );
}

async function main() {
    let alicesKeyPair = await window.crypto.subtle.generateKey(
        {
            name: "ECDH",
            namedCurve: "P-384",
        },
        false,
        ["deriveKey"]
    );
    console.log("Alice's key pair", alicesKeyPair);

    let bobsKeyPair = await window.crypto.subtle.generateKey(
        {
            name: "ECDH",
            namedCurve: "P-384",
        },
        false,
        ["deriveKey"]
    );

    let alicesSecretKey = await deriveSecretKey(
        alicesKeyPair.privateKey,
        bobsKeyPair.publicKey
    );

    let bobsSecretKey = await deriveSecretKey(
        bobsKeyPair.privateKey,
        alicesKeyPair.publicKey
    );

    {
        const message = new TextEncoder().encode("Hello, Bob!");
        const iv = window.crypto.getRandomValues(new Uint8Array(12));
        const ciphertext = await encrypt(message, alicesSecretKey, iv);
        console.log("Alice's ciphertext", ciphertext);
        const decrypted = await decrypt(ciphertext, bobsSecretKey, iv);
        console.log("Bob's decrypted", new TextDecoder().decode(decrypted));
        if (new TextDecoder().decode(decrypted) === "Hello, Bob!") {
            console.log("Alice and Bob can communicate securely!");
        } else {
            throw new Error("Alice and Bob cannot communicate securely!");
        }
    }
}

main()
    .catch(err => { scriptOutput = err; throw err; })
    .finally(process.exit);