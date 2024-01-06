import init, { run } from "./dist/phatjs.js";

// Provide custom fetch implementation for phatjs
window.phatjsFetch = (resource, options) => {
    console.log("Fetch: ", resource, options);
    return fetch(resource, options);
}

async function runScript() {
    const script = document.getElementById("input-code").value;
    document.getElementById("btn-run").disabled = true;
    const args = ["42"];
    try {
        const output = await run(["phatjs", "-c", script, "--", ...args]);
        document.getElementById("output").innerText = output;
    } finally {
        document.getElementById("btn-run").disabled = false;
    }
}

async function main() {
    await init();
    document.getElementById("btn-run").onclick = runScript;
    document.getElementById("btn-run").disabled = false;
}

main().catch(console.error)
