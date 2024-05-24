const repr = Wapo.repr;
const stats = () => repr(Wapo.memoryStats());

class Buffer {
    constructor(capacity) {
        this.buf = new Uint8Array(capacity);
        this.length = 0;
    }
    append(chunk) {
        if (this.length + chunk.length > this.buf.length) {
            const newBuf = new Uint8Array(this.buf.length * 2);
            newBuf.set(this.buf);
            this.buf = newBuf;
        }
        this.buf.set(chunk, this.length);
        this.length += chunk.length;
    }
}

async function main() {
    print("start to get...");
    const response = await fetch("https://files.kvin.wang:8443/tests/16m.txt");
    print("status:", response.status);
    print("statusText:", response.statusText);
    var data = new Buffer(1024 * 1024 * 12);
    for await (const chunk of response.body) {
        data.append(chunk);
        print("Got chunk of size:", chunk.length, "total:", data.length);
        Wapo.inspect("Memory usage:", Wapo.memoryStats());
        print();
    }
    print("Done! data:", data.length);
    Wapo.inspect("Memory usage:", Wapo.memoryStats());
}

main().catch(console.error).finally(Wapo.exit);
