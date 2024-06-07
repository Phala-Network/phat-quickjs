import { ChatOpenAI } from "@langchain/openai";
import { ChatPromptTemplate } from "@langchain/core/prompts";
import { RecursiveCharacterTextSplitter } from "langchain/text_splitter";
import { MemoryVectorStore } from "langchain/vectorstores/memory";
import { createStuffDocumentsChain } from "langchain/chains/combine_documents";
import { createRetrievalChain } from "langchain/chains/retrieval";
import { MinimaxEmbeddings } from "@langchain/community/embeddings/minimax";

import { compile } from "html-to-text";
import { RecursiveUrlLoader } from "@langchain/community/document_loaders/web/recursive_url";


const OPENAI_API_KEY = process.env.OPENAI_API_KEY;
const OPENAI_BASE_URL = process.env.OPENAI_BASE_URL || "https://api.openai.com";
const MINIMAX_API_KEY = process.env.MINIMAX_API_KEY;
const MINIMAX_GROUP_ID = process.env.MINIMAX_GROUP_ID;

async function main() {
    const docsUrl = "https://files.kvin.wang:8443/test-docs/";

    console.log("compiling html-to-text");
    const htmlToText = compile({ wordwrap: 130 }); // returns (text: string) => string;
    const loader = new RecursiveUrlLoader(docsUrl, {
        extractor: doc => {
            if (doc.trimStart().startsWith("<!DOCTYPE html>")) {
                return htmlToText(doc)
            } else {
                return doc;
            }
        },
        maxDepth: 1,
        excludeDirs: ["about:"],
        preventOutside: true,
    });

    console.log("loading docs");
    const docs = await loader.load();
    console.log("loaded docs, length:", docs.length);
    console.log("page 0 length:", docs[0].pageContent.length);

    const splitter = new RecursiveCharacterTextSplitter();
    const splitDocs = await splitter.splitDocuments(docs);
    console.log("split docs, length:", splitDocs.length);
    console.log("split page 0 length:", splitDocs[0].pageContent.length);

    const embeddings = new MinimaxEmbeddings({
        apiKey: MINIMAX_API_KEY,
        minimaxGroupId: MINIMAX_GROUP_ID,
    });

    console.log("embedding docs and creating vector store");
    const vectorstore = await MemoryVectorStore.fromDocuments(
        splitDocs,
        embeddings
    );

    const prompt =
        ChatPromptTemplate.fromTemplate(`Answer the following question based only on the provided context:

<context>
{context}
</context>

Question: {input}`);

    const chatModel = new ChatOpenAI({
        modelName: "gpt-4o",
        openAIApiKey: OPENAI_API_KEY,
        configuration: {
            baseURL: OPENAI_BASE_URL,
        }
    });

    console.log("creating document chain");
    const documentChain = await createStuffDocumentsChain({
        llm: chatModel,
        prompt,
    });

    const retriever = vectorstore.asRetriever();

    console.log("creating retrieval chain");
    const retrievalChain = await createRetrievalChain({
        combineDocsChain: documentChain,
        retriever,
    });

    const question = "Give me the code to calculate sha256 hash of `Hello, world` using wapo js.";
    console.log("==================[Question]====================");
    console.log(question);
    console.log("================================================");
    const result = await retrievalChain.invoke({
        input: question,
    });
    console.log("===================[Answer]=====================");
    console.log("Answer:");
    console.log(result.answer);
    console.log("================================================");

    if (process.env.LANGCHAIN_TRACING_V2 == "true") {
        console.log("\n\nGive some time for langsmith to report the tracing data.");
        await new Promise(resolve => setTimeout(resolve, 5000));
    }
}

main().catch(console.error).finally(() => process.exit());
