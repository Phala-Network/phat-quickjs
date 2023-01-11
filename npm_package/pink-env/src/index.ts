/** Bytes represented in Uint8Array or hex string */
type Bytes = Uint8Array | string;

type Headers = { [key: string]: string };

declare global {
  /** The input arguments passed to the contract eval */
  var scriptArgs: string[];
  /** The extension object for pink contract */
  var pink: {
    /**
     * Call into a contract.
     * @param {Bytes} args.callee - The address of the contract to be called.
     * @param {(number|bigint)} args.gasLimit - The gas limit for the contract call. Defaults to 0;
     * @param {(number|bigint)} args.value - The amount of balance to transfer to the contract. Defaults to 0;
     * @param {number} args.selector - The selector of the ink message to be called.
     * @param {Bytes} args.input - The input arguments for the contract call, encoded in scale.
     * @return {Uint8Array} - The result of the contract call.
     */
    invokeContract(args: {
      callee: Bytes;
      gasLimit?: number | bigint;
      value?: number | bigint;
      selector: number;
      input: Bytes;
    }): Uint8Array;
    /**
     * Invokes a delegate call on a contract code by a code hash.
     * @param {Bytes} args.codeHash - The code hash of the contract to delegate to.
     * @param {number} args.selector - The selector of the ink message to be called.
     * @param {Bytes} args.input - The input arguments for the delegate call, encoded in scale.
     * @return {Uint8Array} - The result of the delegate call.
     */
    invokeContractDelegate(args: {
      codeHash: Bytes;
      selector: number;
      input: Bytes;
    }): Uint8Array;
    /**
     * This function sends an HTTP request and returns the response as either a Uint8Array or a string.
     * @param {string} args.url - The URL to send the request to.
     * @param {string} args.method - The HTTP method to use for the request (e.g. GET, POST, PUT). Defaults to GET.
     * @param {Headers} args.headers - An map-like object containing the headers to send with the request.
     * @param {(Uint8Array|string)} args.body - The body of the request, either as a Uint8Array or a string.
     * @param {boolean} args.returnTextBody - A flag indicating whether the response body should be returned as a string (true) or a Uint8Array (false).
     * @return {Object} - The response from the HTTP request containing the following fields:
     *  - {number} statusCode - The HTTP status code of the response.
     *  - {string} reasonPhrase - The reason phrase of the response.
     *  - {Headers} headers - An object containing the headers of the response.
     *  - {(Uint8Array|string)} body - The response body, either as a Uint8Array or a string depending on the value of args.returnTextBody.
     */
    httpRequest(args: {
      url: string;
      method?: string;
      headers?: Headers;
      body?: Uint8Array | string;
      returnTextBody?: boolean;
    }): {
      statusCode: number;
      reasonPhrase: string;
      headers: Headers;
      body: Uint8Array | string;
    };
  };
}
export {};
