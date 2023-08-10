const { Configuration, OpenAIApi } = require("openai");

const OPENAI_API_KEY = "YOUR_API_KEY";

async function main() {
    console.log("starting...");
    const configuration = new Configuration({
        apiKey: OPENAI_API_KEY,
        formDataCtor: () => {},
    });
    const openai = new OpenAIApi(configuration);
    const chat_completion = await openai.createChatCompletion({
        model: "gpt-3.5-turbo",
        messages: [{ role: "user", content: "Hello world" }],
    });
    console.log("done", chat_completion);
}

main().catch(console.error)