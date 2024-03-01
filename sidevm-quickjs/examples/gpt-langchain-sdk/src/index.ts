import "@phala/sidevm-env"
import { ChatOpenAI } from "@langchain/openai";

const OPENAI_API_KEY = "";

async function main() {
    const chatModel = new ChatOpenAI({
        openAIApiKey: OPENAI_API_KEY
    });
    const response = await chatModel.invoke("what is LangSmith?");
    console.log("response:", JSON.stringify(response));
}

main().catch(console.error).finally(() => Sidevm.exit());
