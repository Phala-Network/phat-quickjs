const OPENAI_API_KEY = "";

function encode(o) {
  return new TextEncoder().encode(JSON.stringify(o))
}

const headers = {
  'Accept': 'application/json, text/plain, */*',
  'Content-Type': 'application/json',
  'Authorization': `Bearer ${OPENAI_API_KEY}`,
};

const body = encode(
  {
    "model": "gpt-3.5-turbo",
    "messages": [{ "role": "user", "content": "Tell me a story" }],
    "stream": true
  }
);

async function main() {
  const response = await fetch("https://api.openai.com/v1/chat/completions", {
    method: 'POST',
    headers,
    body
  });
  // TODO: this should be wrapped in a line-based reader
  for await (const chunk of response.body) {
    console.log('chunk:', new TextDecoder().decode(chunk));
  }
}

main().catch(console.error)

