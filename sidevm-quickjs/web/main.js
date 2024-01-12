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
    } finally {
        setRunable(true);
    }
}

async function main() {
    await init();

    const useProxyCheckbox = document.getElementById('use-proxy');
    const proxyUrlInput = document.getElementById('proxy-url');

    useProxyCheckbox.addEventListener('change', (event) => {
        proxyUrlInput.disabled = !event.target.checked;
    });

    // Provide custom fetch implementation for phatjs
    setHook("fetch", (req) => {
        if (useProxyCheckbox.checked) {
            const proxyUrl = proxyUrlInput.value.trim();
            if (proxyUrl) {
                let url = new URL(proxyUrl);
                url.pathname += req.url;
                req = new Request(url, req);
            }
        }
        return fetch(req);
    });

    setRunable(true, runScript);
}

main().catch(console.error)
