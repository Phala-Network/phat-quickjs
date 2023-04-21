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
 * Represents a SCALE encoder.
 * @typedef ScaleEncoder
 */
export type ScaleEncoder = {
  [_opaqueBrand]: "ScaleEncoder";
};

/**
 * Represents a SCALE decoder.
 * @typedef ScaleDecoder
 */
export type ScaleDecoder = {
  [_opaqueBrand]: "ScaleDecoder";
};

/**
 * Represents a SCALE codec for encoding and decoding data.
 * @interface ScaleCodec
 */
export interface ScaleCodec {
  /**
   * Parses a multi-line string representing types and returns a TypeRegistry.
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
   * Creates a SCALE encoder for encoding tuple values.
   * @function createTupleEncoder
   * @param {number[]} typeIds - An array of type IDs.
   * @param {TypeRegistry} typeRegistry - A TypeRegistry containing the types to be encoded.
   * @returns {ScaleEncoder} - A ScaleEncoder for encoding tuple values.
   */
  createTupleEncoder(
    typeIds: number[],
    typeRegistry: TypeRegistry
  ): ScaleEncoder;

  /**
   * Creates a SCALE encoder for a specific type ID.
   * @function createEncoderForTypeId
   * @param {number} typeId - The type ID for which to create the encoder.
   * @param {TypeRegistry} typeRegistry - A TypeRegistry containing the types to be encoded.
   * @returns {ScaleEncoder} - A ScaleEncoder for encoding values of the specified type ID.
   */
  createEncoderForTypeId(
    typeId: number,
    typeRegistry: TypeRegistry
  ): ScaleEncoder;

  /**
   * Creates a SCALE decoder for decoding tuple values.
   * @function createTupleDecoder
   * @param {number[]} typeIds - An array of type IDs.
   * @param {TypeRegistry} typeRegistry - A TypeRegistry containing the types to be decoded.
   * @returns {ScaleDecoder} - A ScaleDecoder for decoding tuple values.
   */
  createTupleDecoder(
    typeIds: number[],
    typeRegistry: TypeRegistry
  ): ScaleDecoder;

  /**
   * Creates a SCALE decoder for a specific type ID.
   * @function createDecoderForTypeId
   * @param {number} typeId - The type ID for which to create the decoder.
   * @param {TypeRegistry} typeRegistry - A TypeRegistry containing the types to be decoded.
   * @returns {ScaleDecoder} - A ScaleDecoder for decoding values of the specified type ID.
   */
  createDecoderForTypeId(
    typeId: number,
    typeRegistry: TypeRegistry
  ): ScaleDecoder;

  /**
   * Encodes a value using the provided SCALE encoder.
   * @function encode
   * @param {*} value - The value to be encoded.
   * @param {ScaleEncoder} encoder - The ScaleEncoder to use for encoding the value.
   * @returns {Uint8Array} - The encoded value as a Uint8Array.
   */
  encode(value: any, encoder: ScaleEncoder): Uint8Array;

  /**
   * Decodes a value from a Uint8Array using the provided SCALE decoder.
   * @function decode
   * @param {Uint8Array} bytes - The bytes to be decoded.
   * @param {ScaleDecoder} decoder - The ScaleDecoder to use for decoding the bytes.
   * @returns {*} - The decoded value
   */
  decode(bytes: Uint8Array, decoder: ScaleDecoder): any;
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

    /**
     * The SCALE codec object for encoding and decoding data.
     * @typedef SCALE
     * @type {ScaleCodec}
     */
    SCALE: ScaleCodec;
  };
}
export {};