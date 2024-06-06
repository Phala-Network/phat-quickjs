import { Provider, SigningKey, ethers } from "ethers";

async function sleep(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

async function createWallet(key: string | SigningKey, provider: Provider): Promise<ethers.Wallet> {
    await sleep(0);
    const wallet = new ethers.Wallet(key);
    await sleep(0);
    return wallet;

}

async function main() {
    const provider = new ethers.JsonRpcProvider("https://polygon-mumbai.api.onfinality.io/public");
    const block = await provider.getBlockNumber();
    console.log("Current block number: " + block);

    const abi = [
        "function name() view returns (string)",
        "function symbol() view returns (string)",
        "function balanceOf(address) view returns (uint)",
        "function transfer(address to, uint amount)",
        "event Transfer(address indexed from, address indexed to, uint amount)"
    ];
    const contractAddress = "0xA02f6adc7926efeBBd59Fd43A84f4E0c0c91e832";
    const contract = new ethers.Contract(contractAddress, abi, provider);

    const name = await contract.name()
    console.log("Name   :", name);
    const symbol = await contract.symbol()
    console.log("Symbol :", symbol);
    const balance = await contract.balanceOf("0x0000000000000000000000000000000000001010")
    console.log("Balance:", balance);
}

main().catch(console.error).finally(() => process.exit());
