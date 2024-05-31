(function (openai) {
    'use strict';

    const OPENAI_API_KEY = process.env.GPT_API_KEY;
    const BASE_URL = process.env.GPT_BASE_URL || "https://api.openai.com";

    async function main() {
        if (!OPENAI_API_KEY) {
            console.error("Please set the GPT_API_KEY environment variable");
            return;
        }
        console.log("BASE_URL:", BASE_URL);
        const chatModel = new openai.ChatOpenAI({
            openAIApiKey: OPENAI_API_KEY,
            configuration: {
                baseURL: BASE_URL,
            }
        });
        const stream = await chatModel.stream("Hello! Tell me about yourself.");
        for await (const chunk of stream) {
            console.log(`${chunk.content}|`);
        }
    }

    main().catch(console.error).finally(() => process.exit());

})(openai);
