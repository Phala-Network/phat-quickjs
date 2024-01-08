import init, { run, setHook } from "./dist/phatjs.js";

function setRunable(enabled, runner) {
    document.getElementById("btn-run").disabled = !enabled;
    if (runner) {
        document.getElementById("btn-run").onclick = runner;
    }
}

function setOutput(text) {
    document.getElementById("output").value = text;
}

async function runScript() {
    const script = document.getElementById("input-code").value;
    const args = ["42"];
    try {
        setRunable(false);
        setOutput("Running...");
        const output = await run(["phatjs", "-c", script, "--", ...args]);
        setOutput(output);
    } catch (error) {
        setOutput(error);
    }
    finally {
        setRunable(true);
    }
}

async function main() {
    await init();

    // Provide custom fetch implementation for phatjs
    setHook("fetch", (req) => {
        // req = new Request("http://localhost:3000/" + req.url, req);
        return fetch(req);
    });
    setRunable(true, runScript);
}

main().catch(console.error)
