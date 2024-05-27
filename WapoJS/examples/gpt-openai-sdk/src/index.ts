import { Configuration, OpenAIApi } from "openai";

const OPENAI_API_KEY = "";

async function main() {
    const configuration = new Configuration({
        apiKey: OPENAI_API_KEY,
        formDataCtor: Object,
    });
    const openai = new OpenAIApi(configuration);
    const response = await openai.createChatCompletion({
        model: "gpt-3.5-turbo",
        messages: [{ role: "user", content: "Tell me a story" }],
    });
    console.log('response:', JSON.stringify(response.data));
}

main().catch(console.error).finally(() => process.exit());
