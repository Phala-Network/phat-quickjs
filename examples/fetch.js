console.log("Hello, world!");
const chunks = [];
async function test() {
    console.log("entered test");
    const response = await fetch("https://httpbin.org/bytes/4096");
    console.log("status:", response.status);
    console.log("statusText:", response.statusText);
    for await (const chunk of response.body) {
        print("Got chunk of size:", chunk.length);
    }
    print("Done!")
}
test()
