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
import { SidevmDeployer } from "@/typings/SidevmDeployer";
import { Axios } from "axios";
import { Jssrv } from '@/typings/Jssrv';

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

describe("Run tests", () => {
  let system: System.Contract;
  let deployerFactory: SidevmDeployer.Factory;
  let deployer: SidevmDeployer.Contract;
  let jssrvFactory: Jssrv.Factory;
  let jssrv: Jssrv.Contract;
  let pruntime: InstanceType<typeof PhalaSdk.PhactoryAPI>;

  let api: ApiPromise;
  let alice: KeyringPair;
  let certAlice: PhalaSdk.CertificateData;
  const txConf = { gasLimit: "10000000000000", storageDepositLimit: null };
  let currentStack: string;
  const engineCode = fs.readFileSync("../sidejs.wasm");
  const engineCodeHash = blake2AsHex(engineCode);

  before(async function () {
    this.timeout(500_000_000);

    currentStack = this.devPhase.runtimeContext.paths.currentStack;
    console.log("clusterId:", this.devPhase.mainClusterId);
    console.log(`currentStack: ${currentStack}`);

    api = this.api;
    system = (await this.devPhase.getSystemContract(this.devPhase.mainClusterId)) as any;
    console.log("system contract:", system.address.toHex());

    deployerFactory = await this.devPhase.getFactory(`${currentStack}/sidevm_deployer.contract`, { contractType: ContractType.InkCode });
    jssrvFactory = await this.devPhase.getFactory('jssrv', { contractType: ContractType.InkCode });

    await deployerFactory.deploy();
    await jssrvFactory.deploy();

    alice = this.devPhase.accounts.alice;
    certAlice = await PhalaSdk.signCertificate({
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
    jssrv = await jssrvFactory.instantiate("new", [engineCodeHash], txConf as any);
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
        jssrv.address.toHex(),
      ),
      alice,
      true,
    );

    await checkUntil(async () => {
      const { output } = await deployer.query["sidevmOperation::canDeploy"](alice.address, { cert: certAlice }, jssrv.address.toHex());
      return output.asOk.valueOf();
    }, 1000 * 10);
    console.log("sidevmOperation::canDeploy checked");
    pruntime = PhalaSdk.createPruntimeClient(this.devPhase.workerUrl);
  });

  describe("Test quickjs in sidevm", function () {
    this.timeout(500_000_000);

    it("can config the contract", async function () {
      await TxHandler.handle(
        jssrv.tx.setScript(
          { gasLimit: "10000000000000" },
          fs.readFileSync("../examples/httpListen.js", "utf-8"),
        ),
        alice,
        true,
      );
      await TxHandler.handle(
        jssrv.tx.updateConfig(
          { gasLimit: "10000000000000" },
          2
        ),
        alice,
        true,
      );

      // Wait for the config to be updated
      await checkUntil(async () => {
        const config = await jssrv.query.getConfig(alice.address, { cert: certAlice });
        return config.output.eq({ Ok: 2 });
      }, 1000 * 10);
    });

    it("can start sidevm", async function () {
      await jssrv.query.restartSidevm(alice.address, { cert: certAlice });
      await pruntime.uploadSidevmCode({
        contract: jssrv.address.toU8a(),
        code: engineCode,
      })
      await checkUntil(async () => {
        const info = await pruntime.getContractInfo({ contracts: [jssrv.address.toHex()] });
        return info.contracts[0].sidevm?.state === "running";
      }, 1000 * 10);
      await sleep(500);
    });

    it("should be listening HTTP requests in JS", async function () {
      const response = await new Axios({}).get(`${this.devPhase.workerUrl}/sidevm/${jssrv.address.toHex()}/_/`);
      assertTrue(response.status === 200);
      assertTrue(response.headers['x-foo'] === 'Bar');
    });

    it("can update config", async function () {
      const newConfig = 42;
      await TxHandler.handle(
        jssrv.tx.updateConfig(
          { gasLimit: "10000000000000" },
          newConfig
        ),
        alice,
        true,
      );
      await checkUntil(async () => {
        const response = await new Axios({}).get(`${this.devPhase.workerUrl}/sidevm/${jssrv.address.toHex()}/_/getConfig`);
        assertTrue(response.status === 200);
        return response.data === '' + newConfig;
      }, 1000 * 5);
    });
  });
});

function assertTrue(value: boolean) {
  if (!value) {
    throw new Error("Assertion failed");
  }
}
