import { Hono } from 'hono'
const app = new Hono()

app.get('/', (c) => c.text('Hono!'))
app.post('/eval', async (c) => {
    const req = c.req;
    const code = await req.text();
    console.log('source code:', code);
    try {
        const output = await isolateEval(code);
        console.log('output:', output);
        c.status(200);
        return c.json({
            output,
        });
    } catch (e) {
        c.status(500);
        return c.json({ error: e.message });;
    }
});

async function isolateEval(script: string): Promise<any> {
    return new Promise((resolve) => {
        Wapo.isolateEval({
            scripts: [script],
            args: [],
            env: {},

            timeLimit: 60000, // 60s
            gasLimit: 100000, // Necessary to avoid infinite loops
            memoryLimit: 1024 * 1024 * 5,
            polyfills: ["browser"], // Enable browser API polyfills. Can use `["nodejs"]` if needed.
        }, (output) => {
            resolve(output);
        });
    });
}

export default app
