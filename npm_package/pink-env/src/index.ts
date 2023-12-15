/** Bytes represented in Uint8Array or hex string */
type Bytes = Uint8Array | string;

type Headers = { [key: string]: string };

declare const _opaqueBrand: unique symbol;

/**
 * Represents a registry of types.
 * @typedef TypeRegistry
 */
export type TypeRegistry = {
  [_opaqueBrand]: "TypeRegistry";
};

/**
 * Represents a SCALE coder.
 * @typedef Codec
 */
export type Codec = {
  encode: (value: any) => Uint8Array;
  decode: (value: Uint8Array) => any;
};

/**
 * Represents a SCALE codec for encoding and decoding data.
 * @interface ScaleCodec
 */
export interface ScaleCodec {
  /**
   * Parses a multi-line string representing types and returns a TypeRegistry.
   *
   * @function parseTypes
   * @param {string} types - A string representing types.
   * @returns {TypeRegistry} - A TypeRegistry containing the parsed types.
   * @example
   * const typesString = `
   * #bool
   * <Ok:2,Err:3>
   * ()
   * <CouldNotReadInput::1>
   * `;
   * const typeRegistry = parseTypes(typesString);
   */
  parseTypes(types: string): TypeRegistry;

  /**
   * Creates a SCALE codec object for a specific type ID.
   *
   * @function codec
   * @param {number|number[]} typeId - The type ID for which to create the encoder.
   * @param {TypeRegistry} typeRegistry - A TypeRegistry containing the types to be encoded.
   * @returns {Codec} - A ScaleEncoder for encoding values of the specified type ID.
   */
  codec(typeId: number | number[], typeRegistry: TypeRegistry): Codec;

  /**
   * Creates a SCALE codec object for a specific type ID. JsDelegate2 only.
   *
   * @function codec
   * @param {string} typeName - The type name for which to create the encoder.
   * @param {TypeRegistry} typeRegistry - A TypeRegistry containing the types to be encoded.
   * @returns {Codec} - A ScaleEncoder for encoding values of the specified type ID.
   */
  codec(typeName: string, typeRegistry: TypeRegistry): Codec;

  /**
   * SCALE encode an object to bytes. JsDelegate2 only.
   *
   * @function encode
   * @param {string} type - The type name or literal type definition.
   * @param {TypeRegistry} typeRegistry - A TypeRegistry containing the types to be referenced by type.
   * @returns {Uint8Array} - The encoded value.
   */
  encode(data: any, type: string, typeRegistry?: TypeRegistry): Uint8Array;

  /**
   * Decodes a SCALE encoded value. JsDelegate2 only.
   *
   * @function decode
   * @param {Uint8Array} data - The SCALE encoded value.
   * @param {string} type - The type name or literal type definition.
   * @param {TypeRegistry} typeRegistry - A TypeRegistry containing the types to be referenced by type.
   * @returns {any} - The decoded value.
   */
  decode(data: Uint8Array, type: string, typeRegistry?: TypeRegistry): any;
}

declare global {
  /** The input arguments passed to the contract eval */
  var scriptArgs: string[];
  /**
   * The extension object for pink contract.
   * @typedef pink
   */
  var pink: {
    /**
     * Call into a contract.
     * @param {Bytes} args.callee - The address of the contract to be called.
     * @param {(number|bigint)} args.gasLimit - The gas limit for the contract call. Defaults to 0;
     * @param {(number|bigint)} args.value - The amount of balance to transfer to the contract. Defaults to 0;
     * @param {number} args.selector - The selector of the ink message to be called.
     * @param {Bytes} args.input - The input arguments for the contract call, encoded in scale.
     * @param {boolean} args.allowReentry - A flag indicating whether reentry to this contract is allowed. Defaults to false.
     * @return {Uint8Array} - The result of the contract call.
     */
    invokeContract(args: {
      callee: Bytes;
      gasLimit?: number | bigint;
      value?: number | bigint;
      selector: number;
      input: Bytes;
      allowReentry?: boolean;
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

    batchHttpRequest(
      args: {
        url: string;
        method?: string;
        headers?: Headers;
        body?: Uint8Array | string;
        returnTextBody?: boolean;
      }[],
      timeout_ms?: number
    ): {
      statusCode: number;
      reasonPhrase: string;
      headers: Headers;
      body: Uint8Array | string;
      error?: string;
    }[];

    /**
     * Derives a secret key from a salt.
     */
    deriveSecret(salt: Uint8Array | string): Uint8Array;

    /**
     * Hashes a message using the specified algorithm.
     * @param {string} algrithm - The name of the hash algorithm to use.
     *    Supported values are "blake2b128", "blake2b256", "sha256", "keccak256".
     * @param {(Uint8Array|string)} message - The message to hash, either as a Uint8Array or a string.
     */
    hash(algrithm: string, message: Uint8Array | string): Uint8Array;

    /**
     * The SCALE codec object for encoding and decoding data.
     * @typedef SCALE
     * @type {ScaleCodec}
     */
    SCALE: ScaleCodec;

    /**
     * The version of the pink qjs engine.
     */
    version: string;
  };
}

/**
 * Generates 64 bytes of verifiable random bytes.
 *
 * When called in the same contract and same js code with the same salt, the same random bytes
 * will be generated. Different contracts or different js code with the same salt will generate
 * different random bytes.
 *
 * @param {Uint8Array | string} salt - The salt used for generating verifiable random bytes or hex representation.
 * @returns {Uint8Array} - The generated random bytes.
 */
function vrf(salt: Uint8Array | string): Uint8Array {
  function prefixedWith(prefix: string, salt: Uint8Array | string): string {
    if (typeof salt === 'string') {
      if (salt.startsWith("0x")) {
        salt = salt.slice(2);
      }
    } else {
      salt = Array.from(salt).map(byte => byte.toString(16).padStart(2, '0')).join('');
    }
    return prefix + salt;
  }
  const vrfPrefix = '0x7672663a'; // hex of 'vrf:'
  return pink.deriveSecret(prefixedWith(vrfPrefix, salt));
}

export {
  vrf
};
