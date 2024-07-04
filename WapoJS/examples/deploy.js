#!/usr/bin/env node

const fs = require('fs');
const http = require('http');

class WorkerClient {
    constructor(baseUrl, apiToken) {
        this.baseUrl = baseUrl;
        this.apiToken = apiToken;
    }

    async init() {
        await this.initWorkerIfNot();
    }

    async initWorkerIfNot() {
        console.log('Checking worker info...');
        const workerInfo = await this.rpcCall("Info", {});

        if (!workerInfo.session || workerInfo.session === "0x") {
            console.log('No active session found, initializing worker...');
            await this.rpcCall("WorkerInit", {});
            console.log('Worker initialized.');
        } else {
            console.log('Active session found, worker already initialized.');
        }
    }

    async uploadFile(fileName) {
        console.log('Uploading file:', fileName);
        const data = fs.readFileSync(fileName);
        return await this.rpcCall("BlobPut", {
            hash_algorithm: "sha256",
            body: data.toString('hex')
        });
    }

    async deploy(manifest) {
        return await this.rpcCall("AppDeploy", { manifest });
    }

    async rpcCall(method, params) {
        const url = `${this.baseUrl}/prpc/Operation.${method}?json`;
        const headers = {};
        if (this.apiToken) {
            headers['Authorization'] = `Bearer ${this.apiToken}`;
        }
        const response = await httpPost(url, params, headers);
        return JSON.parse(response);
    }
}

function httpPost(url, jsonData, headers = {}) {
    return new Promise((resolve, reject) => {
        const data = JSON.stringify(jsonData);
        const { hostname, pathname, port } = new URL(url);

        const options = {
            hostname,
            port: port || 80,
            path: pathname,
            method: 'POST',
            headers: {
                ...headers,
                'Content-Type': 'application/json',
                'Content-Length': data.length
            }
        };

        const req = http.request(options, (res) => {
            let responseData = '';
            res.on('data', chunk => responseData += chunk);
            res.on('end', () => {
                if (res.statusCode >= 200 && res.statusCode < 300) {
                    resolve(responseData);
                } else {
                    const errorMsg = `HTTP status code ${res.statusCode}: ${responseData}`;
                    reject(new Error(errorMsg));
                }
            });
        });

        req.on('error', error => {
            reject(error);
        });
        req.write(data);
        req.end();
    });
}

async function main() {
    const WAPOD_URL = process.env.WAPOD_URL || "http://127.0.0.1:8001";
    const API_TOKEN = process.env.WAPOD_API_TOKEN;
    const engineFile = process.argv[2];
    const scriptFile = process.argv[3];
    if (!engineFile || !scriptFile) {
        console.error('Usage: deploy.js <engine_wasm_file> <js_script_file>');
        console.error('Please provide both engine and script file names as arguments.');
        process.exit(1);
    }

    try {
        const client = new WorkerClient(WAPOD_URL, API_TOKEN);
        await client.init();

        const engineInfo = await client.uploadFile(engineFile);
        const scriptInfo = await client.uploadFile(scriptFile);

        const manifest = {
            version: 1,
            code_hash: engineInfo.hash,
            hash_algorithm: "sha256",
            args: ["--code-hash", scriptInfo.hash],
            env_vars: [],
            on_demand: false,
            resizable: false,
            max_query_size: 10240,
            label: "GPT Proxy",
        };

        console.log('Deploying app...');
        const appInfo = await client.deploy(manifest);
        console.log('App deployed, address is', appInfo.address);
    } catch (error) {
        console.error('An error occurred during the main process:', error);
    }
}

main().catch(console.error);
