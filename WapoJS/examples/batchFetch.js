async function task(i) {
    console.log(i, "start to get...");
    const response = await fetch("https://files.kvin.wang:8443/tests/1m.txt");
    console.log(i, "status:", response.status);
    console.log(i, "statusText:", response.statusText);
    for await (const chunk of response.body) {
        print(i, "Got chunk of size:", chunk.length);
    }
    return "done!"
}

Promise.all([task(0), task(1), task(2), task(3)])
    .then(x => scriptOutput = x.join("|"))
    .catch(console.error)
    .finally(process.exit);
