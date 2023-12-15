async function main() {
    const url = 'https://httpbin.org/get';
    console.log("getting url:", url);
    const response = await fetch(url);
    globalThis.scriptOutput = await response.text();
    console.log("scriptOutput:", scriptOutput);
}

main().catch(console.error);
