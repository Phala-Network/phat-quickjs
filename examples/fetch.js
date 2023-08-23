const chunks = [];
async function test_get() {
    console.log("start to get...");
    const response = await fetch("https://httpbin.kvin.wang:8443/bytes/4096");
    console.log("status:", response.status);
    console.log("statusText:", response.statusText);
    for await (const chunk of response.body) {
        print("Got chunk of size:", chunk.length);
        console.log("chunk:", new TextDecoder().decode(chunk));
    }
    print("Done!")
}
async function test_post() {
    console.log("start to post...");
    const response = await fetch("https://httpbin.kvin.wang:8443/post", {
        method: "POST",
        body: "0x303132",
    });
    console.log("status:", response.status);
    console.log("statusText:", response.statusText);
    const body = await response.text();
    console.log("resposne.body:", body);
    print("Done!")
}
async function test_all() {
    await test_get();
    await test_post();
}

test_all()
