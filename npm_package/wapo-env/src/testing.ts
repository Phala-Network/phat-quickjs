import type { TypeRegistry, Codec, LockGuard, ReadableStreamHandle, WriteableStreamHandle, WriteStream, DataCallback, BoolCallback, HttpsConfig, HttpConfig, HttpResponseHeadHandle, QueryResposneHandle, Query, IsolateEvalArgs, RunCodeOptions, RunCodeReturns, MemoryStats, HttpResponseHead, IncomingRequest, HttpRequestReceipt, HttpRequest, ClientHttpResponseHead } from './index'
import { blake2b } from '@noble/hashes/blake2b'
import { keccak_256, sha3_256 } from '@noble/hashes/sha3'
import { Wyhash } from 'wyhash.js'

function hash(algrithm: 'blake2b128' | 'blake2b256' | 'blake2b512' | 'sha256' | 'keccak256', message: Uint8Array | string): Uint8Array {
    if (algrithm === 'blake2b128') {
        return blake2b(message, { dkLen: 16 });
    } else if (algrithm === 'blake2b256') {
        return blake2b(message, { dkLen: 32 });
    } else if (algrithm === 'blake2b512') {
        return blake2b(message, { dkLen: 64 });
    } else if (algrithm === 'sha256') {
        return sha3_256(message);
    } else if (algrithm === 'keccak256') {
        return keccak_256(message);
    } else {
        throw new Error("Unsupported hash algorithm");
    }
}


globalThis.Wapo = {
    version: '0.9.3',

    SCALE: {
        codec: function(typeId: number, typeRegistry: TypeRegistry): Codec {
            throw new Error("Not implemented");
        },
        parseTypes: function(types: string): TypeRegistry {
            throw new Error("Not implemented");
        }
    },

    deriveSecret: function(salt: Uint8Array | string): Uint8Array {
        return hash('blake2b256', salt);
    },

    hash: hash,

    nonCryptographicHash: function(algrithm: 'wyhash64', message: Uint8Array | string): Uint8Array {
        const hasher = new Wyhash(42n);
        const result = hasher.hash(message);
        return new Uint8Array(new BigUint64Array([result]).buffer);
    },

    concatU8a: function(arrays: Uint8Array[]): Uint8Array {
        const totalLength = arrays.reduce((acc, arr) => acc + arr.length, 0);
        const result = new Uint8Array(totalLength);
        let offset = 0;
        for (const arr of arrays) {
            result.set(arr, offset);
            offset += arr.length;
        }
        return result;
    },

    exit: function(): never {
        console.log("Exiting...");
        throw new Error("Exit called");
    },

    inspect: function(obj: any): string {
        return JSON.stringify(obj, null, 2);
    },

    workerSign: function(message: Uint8Array): Uint8Array {
        return new Uint8Array(64);
    },

    workerPublicKey: function(): Uint8Array {
        return new Uint8Array(32);
    },

    sgxQuote: function (message: Uint8Array | string): Uint8Array | undefined {
        return undefined;
    },

    bootData: function(): Uint8Array {
        throw new Error("Not implemented");
    },

    storeBootData: function(data: Uint8Array): void {
        throw new Error("Not implemented");
    },

    tryLock: function(key: string): LockGuard {
        throw new Error("Not implemented");
    },

    unlock: function(key: LockGuard): void {
        throw new Error("Not implemented");
    },

    streamBridge(inputStream: ReadableStreamHandle, outputStream: WriteableStreamHandle): number {
        throw new Error("Not implemented");
    },

    streamOpenWrite(outputStream: WriteableStreamHandle): WriteStream {
        throw new Error("Not implemented");
    },

    streamWriteChunk(writer: WriteStream, chunk: Uint8Array, callback: BoolCallback): void {
        throw new Error("Not implemented");
    },

    streamOpenRead(inputStream: ReadableStreamHandle, callback: DataCallback): number {
        throw new Error("Not implemented");
    },

    streamClose(writer: WriteStream): void {
        throw new Error("Not implemented");
    },

    httpRequest(req: HttpRequest, callback: (resp: ClientHttpResponseHead) => any): HttpRequestReceipt {
        throw new Error("Not implemented");
    },

    httpsListen(config: HttpsConfig | HttpConfig, handler: (req: IncomingRequest) => any): void {
        throw new Error("Not implemented");
    },

    httpsSendResponseHead(tx: HttpResponseHeadHandle, response: HttpResponseHead): void {
        throw new Error("Not implemented");
    },

    httpsSendResponseHeadRaw(tx: HttpResponseHeadHandle, head: string): void {
        throw new Error("Not implemented");
    },

    queryListen(callback: (query: Query) => void): void {
        throw new Error("Not implemented");
    },

    queryReply(tx: QueryResposneHandle, data: Uint8Array | string): void {
        throw new Error("Not implemented");
    },

    isolateEval(args: IsolateEvalArgs, callback: (output: string | Uint8Array | undefined) => unknown): number {
        throw new Error("Not implemented");
    },

    run<Value = unknown>(code: string, options?: RunCodeOptions): Promise<RunCodeReturns<Value>> {
        throw new Error("Not implemented");
    },

    memoryStats(): MemoryStats {
        throw new Error("Not implemented");
    },
}
