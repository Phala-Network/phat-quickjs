import { RuntimeContext, RunMode, AbiTypeBindingProcessor, TypeBinder } from "@devphase/service";
import * as fs from 'fs';

function createBinding(contract: string, name: string) {
    const abi = JSON.parse(fs.readFileSync(contract, 'utf-8'));
    const output = `typings/${name}.ts`;
    AbiTypeBindingProcessor.createTypeBindingFile(output, name, abi);
}

async function prepareStack(): Promise<string> {
    const ctx = await RuntimeContext.getSingleton();
    await ctx.initContext(RunMode.Simple);
    await ctx.requestStackBinaries(true);
    return ctx.paths.currentStack;
}

async function main() {
    const stackDir = await prepareStack();
    createBinding(`${stackDir}/system.contract`, 'System');
    createBinding(`${stackDir}/sidevm_deployer.contract`, 'SidevmDeployer');
}

main().then(() => process.exit(0)).catch(console.error).finally(() => process.exit(-1));