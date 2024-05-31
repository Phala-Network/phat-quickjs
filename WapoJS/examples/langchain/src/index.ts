import { ChatOpenAI } from "@langchain/openai";
import { ChatPromptTemplate } from "@langchain/core/prompts";
import { StringOutputParser } from "@langchain/core/output_parsers";

const OPENAI_API_KEY = process.env.GPT_API_KEY;
const BASE_URL = process.env.GPT_BASE_URL || "https://api.openai.com";

async function main() {
    if (!OPENAI_API_KEY) {
        console.error("Please set the GPT_API_KEY environment variable");
        return;
    }
    console.log("BASE_URL:", BASE_URL);

    const prompt = ChatPromptTemplate.fromMessages([
        ["system", "You are a world class technical documentation writer."],
        ["user", "{input}"],
    ]);
    const chatModel = new ChatOpenAI({
        openAIApiKey: OPENAI_API_KEY,
        configuration: {
            baseURL: BASE_URL,
        }
    });
    const outputParser = new StringOutputParser();
    const llmChain = prompt.pipe(chatModel).pipe(outputParser);
    const stream = await llmChain.stream({ input: "Hello! Tell me about yourself." });
    const chunks = [];
    for await (const chunk of stream) {
        chunks.push(chunk);
        console.log(`${chunk}|`);
    }
}

main().catch(console.error).finally(() => process.exit());
