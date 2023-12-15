const hash = Sidevm.hash;
const iterations = 10000;
function bench(hashName, iterations) {
    const t0 = Date.now();
    var digest = "Hello World!";
    for (var i = 0; i < iterations; i++) {
        digest = hash(hashName, digest);
    }
    const finalDigest = Sidevm.hexEncode(digest.slice(0, 4));
    console.log(`${hashName}(${finalDigest}): ${iterations} iterations in ${Date.now() - t0}ms, ${iterations / (Date.now() - t0) * 1000} calc/sec`);
}
bench("sha256", iterations);
bench("keccak256", iterations);
bench("blake2b128", iterations);
bench("blake2b256", iterations);
bench("blake2b512", iterations);
