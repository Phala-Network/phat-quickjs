let script = `
    console.log("in isolate, Before timeout: " + new Date());
    setTimeout(() => {
        console.log("in isolate: Inside timeout: " + new Date());
        scriptOutput = scriptArgs[0] + " env HELLO=" + process.env.HELLO;
    }, 2000);
`;

async function main() {
    const token = Wapo.isolateEval({
        scripts: [script],
        args: ["input 0", "input 1"],
        env: {},

        gasLimit: 1000,
        timeLimit: 3000,
        memoryLimit: 32 * 1024 + 11400,
        polyfills: ['browser'],
    }, function (result) {
        console.log("isolate, scriptOutput: " + result);
    });

    setTimeout(() => {
        Wapo.isolateKill(token);
    }, 1000);
}

main().catch(console.error);