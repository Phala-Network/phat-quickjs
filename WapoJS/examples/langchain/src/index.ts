import { ChatOpenAI } from "@langchain/openai";

const OPENAI_API_KEY = process.env.GPT_API_KEY;
const BASE_URL = process.env.GPT_BASE_URL || "https://api.openai.com";

async function main() {
    if (!OPENAI_API_KEY) {
        console.error("Please set the GPT_API_KEY environment variable");
        return;
    }
    console.log("BASE_URL:", BASE_URL);
    const chatModel = new ChatOpenAI({
        openAIApiKey: OPENAI_API_KEY,
        configuration: {
            baseURL: BASE_URL,
        }
    });
    const stream = await chatModel.stream("Hello! Tell me about yourself.");
    const chunks = [];
    for await (const chunk of stream) {
        chunks.push(chunk);
        console.log(`${chunk.content}|`);
    }
}

main().catch(console.error).finally(() => process.exit());
