import * as fs from 'fs';
import * as PhalaSdk from "@phala/sdk";
import { ApiPromise } from "@polkadot/api";
const { blake2AsHex } = require('@polkadot/util-crypto');
import type { KeyringPair } from "@polkadot/keyring/types";
import {
  TxHandler,
  ContractType
} from "@devphase/service";
import { System } from "@/typings/System";
import { Control } from "@/typings/Control";
import { SidevmDeployer } from "@/typings/SidevmDeployer";
import { Axios } from "axios";

import "dotenv/config";

async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function checkUntil(async_fn, timeout) {
  const t0 = new Date().getTime();
  while (true) {
    if (await async_fn()) {
      return;
    }
    const t = new Date().getTime();
    if (t - t0 >= timeout) {
      throw new Error("timeout");
    }
    await sleep(100);
  }
}

describe("Run lego actions", () => {
  let system: System.Contract;
  let deployerFactory: SidevmDeployer.Factory;
  let deployer: SidevmDeployer.Contract;
  let controlFactory: Control.Factory;
  let control: Control.Contract;
  let pruntime: InstanceType<typeof PhalaSdk.PhactoryAPI>;

  let api: ApiPromise;
  let alice: KeyringPair;
  let certAlice: PhalaSdk.CertificateData;
  const txConf = { gasLimit: "10000000000000", storageDepositLimit: null };
  let currentStack: string;

  before(async function () {
    this.timeout(500_000_000);

    currentStack = this.devPhase.runtimeContext.paths.currentStack;
    console.log("clusterId:", this.devPhase.mainClusterId);
    console.log(`currentStack: ${currentStack}`);

    api = this.api;
    system = (await this.devPhase.getSystemContract(this.devPhase.mainClusterId)) as any;
    console.log("system contract:", system.address.toHex());

    deployerFactory = await this.devPhase.getFactory(`${currentStack}/sidevm_deployer.contract`, { contractType: ContractType.InkCode });
    controlFactory = await this.devPhase.getFactory('control', { contractType: ContractType.InkCode });

    await deployerFactory.deploy();
    await controlFactory.deploy();

    alice = this.devPhase.accounts.alice;
    certAlice = await PhalaSdk.signCertificate({
      api,
      pair: alice,
    });
    console.log("Signer:", alice.address.toString());

    // Upgrade pink runtime to latest, so that we can store larger values to the storage
    await TxHandler.handle(
      system.tx["system::upgradeRuntime"](
        { gasLimit: "10000000000000" },
        [1, 2],
      ),
      alice,
      true,
    );

    console.log("Instantiating contracts...");
    deployer = await deployerFactory.instantiate("default", [], txConf as any);
    control = await controlFactory.instantiate("default", [], txConf as any);
    await TxHandler.handle(
      system.tx["system::grantAdmin"](
        { gasLimit: "10000000000000" },
        deployer.address.toHex(),
      ),
      alice,
      true,
    );
    await TxHandler.handle(
      system.tx["system::setDriver"](
        { gasLimit: "10000000000000" },
        'SidevmOperation',
        deployer.address.toHex(),
      ),
      alice,
      true,
    );
    await TxHandler.handle(
      deployer.tx["allow"](
        { gasLimit: "10000000000000" },
        control.address.toHex(),
      ),
      alice,
      true,
    );

    await checkUntil(async () => {
      const { output } = await deployer.query["sidevmOperation::canDeploy"](alice.address, { cert: certAlice }, control.address.toHex());
      return output.asOk.valueOf();
    }, 1000 * 10);
    console.log("sidevmOperation::canDeploy checked");
    pruntime = PhalaSdk.createPruntimeClient(this.devPhase.workerUrl);
  });

  describe("Test quickjs in sidevm", function () {
    this.timeout(500_000_000);

    it("can start sidevm", async function () {
      const code = fs.readFileSync("../qjs.wasm");
      const codeHash = blake2AsHex(code);
      await control.query.startSidevm(alice.address, { cert: certAlice }, codeHash);
      await pruntime.uploadSidevmCode({
        contract: control.address.toU8a(),
        code,
      })
      await sleep(500);
      await checkUntil(async () => {
        const info = await pruntime.getContractInfo({ contracts: [control.address.toHex()] });
        return info.contracts[0].sidevm?.state === "running";
      }, 1000 * 10);
    });

    it("can query to sidevm from pink", async function () {
      const { output } = await control.query.querySidevm(alice.address, { cert: certAlice }, 'ping');
      assertTrue(output.asOk.eq('pong'));
    });

    it("should be listening HTTP requests in JS", async function () {
      const url = `${this.devPhase.workerUrl}/sidevm/${control.address.toHex()}/_main/`;
      const client = new Axios({});
      const response = await client.get(url);
      assertTrue(response.status === 200);
      assertTrue(response.headers['x-foo'] === 'Bar');
    });
  });
});

function assertTrue(value: boolean) {
  if (!value) {
    throw new Error("Assertion failed");
  }
}
