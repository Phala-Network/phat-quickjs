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
   * @param types - A string representing types.
   * @returns A TypeRegistry containing the parsed types.
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
   * @param typeId - The type ID for which to create the encoder.
   * @param typeRegistry - A TypeRegistry containing the types to be encoded.
   * @returns A ScaleEncoder for encoding values of the specified type ID.
   */
  codec(typeId: number | number[], typeRegistry: TypeRegistry): Codec;
}


declare global {
  /** The input arguments passed to the contract eval */
  var scriptArgs: string[];
  /** The return value of the JS eval. It would override the value last expression of the script. */
  var scriptOutput: any;
  /**
   * The runtime extension object for wapo env.
   */
  var Wapo: {
    /**
     * The version of the wapo js runtime.
     */
    version: string;

    /**
     * The SCALE codec object for encoding and decoding data.
     * @typedef SCALE
     * @type {ScaleCodec}
     */
    SCALE: ScaleCodec;
    /**
     * Derives a secret key from a salt. The same app with the same salt on the same worker will always
     * derive the same secret. However, the same app with the same salt on different workers will derive
     * different secrets.
     */
    deriveSecret(salt: Uint8Array | string): Uint8Array;

    /**
     * Hashes a message using the specified algorithm.
     * @param algrithm - The name of the hash algorithm to use.
     *    Supported values are "blake2b128", "blake2b256", "blake2b512", "sha256", "keccak256"
     * @param message - The message to hash, either as a Uint8Array or a string.
     */
    hash(algrithm: 'blake2b128' | 'blake2b256' | 'blake2b512' | 'sha256' | 'keccak256', message: Uint8Array | string): Uint8Array;

    /**
     * Non-cryptographic hashing, current only supported wyhash64 64-bit hash. Non-cryptographic algorithms
     * are optimized for speed of computation over collision-resistance or seurity.
     *
     * @param algrithm - The name of the hash algorithm to use.
     *    Supported values are "wyhash64"
     * @param message - The message to hash, either as a Uint8Array or a string.
     */
    nonCryptographicHash(algrithm: 'wyhash64', message: Uint8Array | string): Uint8Array;

    /**
     * Concatenates multiple Uint8Array objects into a single Uint8Array.
     *
     * @param arrays - The arrays to concatenate.
     */
    concatU8a(arrays: Uint8Array[]): Uint8Array;

    /**
     * Terminates the script execution.
     */
    exit(): void;

    /**
     * Prints the specified data to the console, recursively.
     */
    inspect(...data: any[]): void;

    /**
     * Signs the provided message using the worker's private key.
     * @param message - The message to sign.
     * @returns The signature.
     */
    workerSign(message: Uint8Array | string): Uint8Array;

    /**
     * Retrieves the worker's public key.
     * @returns The worker's public key (32 bytes).
     */
    workerPublicKey(): Uint8Array;

    /**
     * Generates an SGX quote for the provided message.
     * @param message - The message to include in the quote.
     * @returns The SGX quote or undefined if not available.
     */
    sgxQuote(message: Uint8Array | string): Uint8Array | undefined;

    /**
     * Retrieves the boot data stored in the worker for this app.
     * @returns The boot data, or undefined if not available.
     */
    bootData(): Uint8Array | undefined;
    /**
     * Sets the boot data for the app in the worker.
     * @param data - The boot data to be stored. Max size is 64KB.
     */
    storeBootData(data: Uint8Array | string): void;
    /**
     * Attempts to acquire a lock for the specified key. The lock scope is inatances of current app in the same worker.
     *
     * And app can hold at most 64 locks at the same time.
     *
     * @param key - The key for which to acquire the lock. Max length is 64 bytes.
     * @returns An opaque value representing the lock, or undefined if the lock could not be acquired.
     */
    tryLock(key: string): LockGuard;
    /**
     * Unlocks the previously acquired lock.
     * @param lock - The opaque value representing the lock to be unlocked.
     */
    unlock(lock: LockGuard): void;


    /**
     * Bridges two streams, copying data from the input stream to the output stream.
     * @param inputStream - The input stream to read data from.
     * @param outputStream - The output stream to write data to.
     * @returns The ID of the spawned task.
     */
    streamBridge(inputStream: ReadableStreamHandle, outputStream: WriteableStreamHandle): number;

    /**
     * Creates a writable stream.
     * @param outputStream - The underlying stream to write data to.
     * @returns An opaque object representing the writable stream.
     */
    streamOpenWrite(outputStream: WriteableStreamHandle): WriteStream;

    /**
     * Writes a chunk of data to the writable stream.
     * @param writer - The writable stream to write data to.
     * @param chunk - The chunk of data to write.
     * @param callback - A callback function to be called with the result of the write operation.
     */
    streamWriteChunk(writer: WriteStream, chunk: Uint8Array, callback: BoolCallback): void;

    /**
     * Creates a readable stream.
     * @param inputStream - The underlying stream to read data from.
     * @param callback - A callback function to be called with data or events from the stream.
     * @returns The ID of the spawned task.
     */
    streamOpenRead(inputStream: ReadableStreamHandle, callback: DataCallback): number;

    /**
     * Closes the writable stream.
     * @param writer - The writable stream to close.
     */
    streamClose(writer: WriteStream): void;

    /**
     * Sends an HTTP request.
     * @param req - The HTTP request object.
     * @param callback - A callback function to be called with the response events.
     * @returns An object containing the cancel token and the opaque body stream (if applicable).
     */
    httpRequest(req: HttpRequest, callback: (resp: ClientHttpResponseHead) => any): HttpRequestReceipt;

    /**
     * Starts an HTTP(S) server and listens for incoming connections.
     * @param config - The configuration for the HTTP(S) server.
     * @param handler - A callback function that handles incoming requests.
     */
    httpsListen(config: HttpsConfig | HttpConfig, handler: (req: IncomingRequest) => any): void;

    /**
     * Sends an HTTP response head to the specified transmitter.
     * @param tx - The transmitter to send the response head to.
     * @param response - The HTTP response head to send.
     */
    httpsSendResponseHead(tx: HttpResponseHeadHandle, response: HttpResponseHead): void;

    /**
     * Sends an HTTP response head in raw string format to the specified transmitter.
     * @param tx - The transmitter to send the response head to.
     * @param head - The raw string representation of the HTTP response head.
     */
    httpsSendResponseHeadRaw(tx: HttpResponseHeadHandle, head: string): void;

    /**
     * Sets the query listener callback function.
     * @param callback - The callback function to be called when a query is received.
     */
    queryListen(callback: (query: Query) => void): void;

    /**
     * Sends a reply to a query.
     * @param tx - The opaque object representing the query reply transaction.
     * @param data - The data to send as a reply.
     */
    queryReply(tx: QueryResposneHandle, data: Uint8Array | string): void;

    /**
     * Evaluates scripts in an isolated environment.
     * @param args - The arguments for the isolated evaluation.
     * @param callback - A callback function to be called with the result of the evaluation.
     * @returns The ID of the spawned task.
     */
    isolateEval(args: IsolateEvalArgs, callback: (output: string | Uint8Array | undefined) => unknown): number;

    /**
     * Evaluates scripts in an isolated environment, asynchronously friendly version with pre-defined default options.
     *
     * @param code - The code to be evaluated.
     * @param options - The options for the isolated evaluation.
     * @returns A promise that resolves with the result of the evaluation.
     */
    run<Value = unknown>(code: string, options?: RunCodeOptions): Promise<RunCodeReturns<Value>>;

    /**
     * Retrieves memory statistics for the current runtime.
     * @returns An object containing memory usage statistics.
     */
    memoryStats(): MemoryStats;
  };
}

export type DataCallback = (cmd: string, data: Uint8Array) => any;
export type BoolCallback = (value: boolean, err: string | undefined) => any;


/**
 * Represents an HTTP request.
 */
interface HttpRequest {
  url: string;
  method?: string;
  headers?: HeadersIn;
  body?: string | Uint8Array;
  /**
   * If true, the body is streamed using the returned HttpRequestReceipt.opaqueBodyStream.
   */
  streamBody?: boolean;
}

/**
 * Represents the receipt of an HTTP request.
 */
interface HttpRequestReceipt {
  cancelToken: number;
  opaqueBodyStream?: WriteableStreamHandle;
}

/**
 * Represents the response headers for an HTTP request.
 */
interface ClientHttpResponseHead {
  status: number;
  statusText: string;
  version: string;
  headers: HeadersIn;
  opaqueBodyStream: ReadableStreamHandle;
}

/**
 * Represents a collection of HTTP headers.
 */
type HeadersOut = HeadersIn | Record<string, string>;

/**
 * Represents the configuration for HTTPS server.
 */
export interface HttpsConfig {
  /**
   * The server name indication (SNI) for the TLS connection.
   */
  serverName: string;

  /**
   * The certificate chain in PEM format.
   */
  certificateChain: string;

  /**
   * The private key in PEM format.
   */
  privateKey: string;
}

/**
 * Represents the configuration for bare HTTP server.
 */
export interface HttpConfig {
  /**
   * The address to listen on. (e.g. "localhost:8080")
   */
  address: string;
}

/**
 * Represents the head (status and headers) of an HTTP response.
 */
export interface HttpResponseHead {
  /**
   * The HTTP status code of the response.
   */
  status: number;

  /**
   * The headers of the HTTP response.
   */
  headers: HeadersOut;
}

/**
 * Represents a collection of HTTP headers.
 */
export type HeadersIn = Array<[string, string]>;

/**
 * Represents an HTTP request.
 */
export interface IncomingRequest {
  /**
   * The HTTP method of the request (e.g., GET, POST, PUT, DELETE).
   */
  method: string;

  /**
   * The URL of the request.
   */
  url: string;

  /**
   * The headers of the request.
   */
  headers: HeadersIn;

  /**
   * An opaque value representing the response transmitter.
   */
  opaqueResponseTx: HttpResponseHeadHandle;

  /**
   * An opaque value representing the input stream of the request.
   */
  opaqueInputStream: ReadableStreamHandle;

  /**
   * An opaque value representing the output stream of the request.
   */
  opaqueOutputStream: WriteableStreamHandle;
}

/**
 * Represents a lock acquired through `tryLock`.
 */
interface LockGuard {
  [_lockGuardBrand]: "LockGuard";
}
declare const _lockGuardBrand: unique symbol;

/**
 * Represents a readable stream handle.
 */
export interface ReadableStreamHandle {
  [_readStreamHandleBrand]: "ReadableStreamHandle";
}
declare const _readStreamHandleBrand: unique symbol;

/**
 * Represents a writable stream handle.
 */
export interface WriteableStreamHandle {
  [_writeStreamHandleBrand]: "WriteableStreamHandle";
}
declare const _writeStreamHandleBrand: unique symbol;

/**
 * A handle used to respond to an HTTP request.
 */
export interface HttpResponseHeadHandle {
  [_httpResponseHeadHandleBrand]: "HttpResponseHeadHandle";
}
declare const _httpResponseHeadHandleBrand: unique symbol;
/**
 * A handle used to respond to a query
 */
export interface QueryResposneHandle {
  [_queryResponseHandleBrand]: "QueryResponseHandle";
}
declare const _queryResponseHandleBrand: unique symbol;

/**
 * Represents a writable stream.
 */
export interface WriteStream {
  [_writeStreamBrand]: "WriteStream";
}
declare const _writeStreamBrand: unique symbol;


/**
 * Represents a query received by the query listener.
 */
interface Query {
  /**
   * The origin of the query, if available.
   */
  origin?: Uint8Array;

  /**
   * The path of the query.
   */
  path: string;

  /**
   * The payload of the query.
   */
  payload: Uint8Array;

  /**
   * The opaque object representing the query reply transaction.
   */
  replyTx: QueryResposneHandle;
}

/**
 * Arguments for the `isolateEval` function.
 */
interface IsolateEvalArgs {
  /**
   * The scripts to be evaluated.
   */
  scripts: string[];
  /**
   * The arguments to be passed to the scripts.
   */
  args: string[];
  /**
   * The environment variables to be set in the isolated environment.
   */
  env: Record<string, string>;
  /**
   * The gas limit for the isolated evaluation.
   */
  gasLimit?: number;
  /**
   * The memory limit for the isolated evaluation.
   */
  memoryLimit?: number;
  /**
   * The time limit for the isolated evaluation.
   */
  timeLimit?: number;
  /**
   * The polyfills to be loaded in the isolated environment.
   * 'browser' for browser polyfills, 'nodejs' for nodejs API polyfills.
   */
  polyfills: string[];
}



export type RunCodeOptions = Partial<Omit<IsolateEvalArgs, 'scripts'>>;

export interface RunCodeReturns<Value = unknown> {
  isOk: Readonly<Boolean>;
  isError: Readonly<Boolean>;
  error: Readonly<string>;
  value: Readonly<Value>;
  logs: Readonly<string[]>;
}


/**
 * Represents memory usage statistics.
 */
interface MemoryStats {
  /**
   * The current memory usage.
   */
  current: number;

  /**
   * The peak memory usage after a spike.
   */
  spike: number;

  /**
   * The overall peak memory usage.
   */
  peak: number;
}


export { };
