let script = `
    console.log("in isolate, Before timeout: " + new Date());
    setTimeout(() => {
        console.log("in isolate: Inside timeout: " + new Date());
        scriptOutput = scriptArgs[0] + " env HELLO=" + process.env.HELLO;
    }, 2000);
`;

async function main() {
    const id = Wapo.isolateEval({
        scripts: [script],
        args: ["input 0", "input 1"],
        env: {
            HELLO: "world",
        },

        gasLimit: 1000,
        timeLimit: 3000,
        memoryLimit: 256 * 1024,
        polyfills: ['wapo'],
    }, function (result) {
        console.log("isolate, scriptOutput: " + result);
    });
}

main().catch(console.error);