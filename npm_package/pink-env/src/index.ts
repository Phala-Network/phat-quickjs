/** Bytes represented in Uint8Array or hex string */
type Bytes = Uint8Array | string;

interface ContractCallArgs {
  /** The callee address */
  callee: Bytes;
  /** Gas limit */
  gasLimit: number | bigint;
  /** Amount of balance that transfer to the callee */
  value: number | bigint;
  /** The selector of the ink message to be called */
  selector: number;
  /** Scale encoded input arguments */
  input: Bytes;
}

interface DelegateCallArgs {
  /** The code hash that would delegate to */
  codeHash: Bytes;
  /** The selector of the ink message to be called */
  selector: number;
  /** Scale encoded input arguments */
  input: Bytes;
}

declare global {
  const pink: {
    /** Call into a contract */
    invokeContract(args: ContractCallArgs): Uint8Array;
    /** Delegate call into a piece of code */
    invokeContractDelegate(args: DelegateCallArgs): Uint8Array;
  };
}
export {};
