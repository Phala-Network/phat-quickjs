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
   * @function createEncoderForTypeId
   * @param {number|number[]} typeId - The type ID for which to create the encoder.
   * @param {TypeRegistry} typeRegistry - A TypeRegistry containing the types to be encoded.
   * @returns {Codec} - A ScaleEncoder for encoding values of the specified type ID.
   */
  codec(typeId: number | number[], typeRegistry: TypeRegistry): Codec;
}

declare global {
  /** The input arguments passed to the contract eval */
  var scriptArgs: string[];
  /** The return value of the JS eval. It would override the value last expression of the script. */
  var scriptOutput: any;
  /**
   * The runtime extension object for sidevm env.
   */
  var Sidevm: {
    /**
     * The SCALE codec object for encoding and decoding data.
     * @typedef SCALE
     * @type {ScaleCodec}
     */
    SCALE: ScaleCodec;
  };
}
export {};
