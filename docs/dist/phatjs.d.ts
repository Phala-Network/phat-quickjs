/* tslint:disable */
/* eslint-disable */
/**
*/
export function start(): void;
/**
* Get the version of the runtime.
* @returns {Promise<string>}
*/
export function version(): Promise<string>;
/**
* Run a script.
*
* # Arguments
* - `args` - a list of arguments to pass to the runtime, including the script name and arguments.
*
* # Example
*
* ```js
* const result = await run(["phatjs", "-c", "console.log(scriptArgs)", "--", "Hello, world!"]);
* console.log(result);
* ```
* @param {(string)[]} args
* @returns {Promise<any>}
*/
export function run(args: (string)[]): Promise<any>;
/**
* Set a hook for the runtime.
*
* # Available hooks
* - `fetch` - a function that takes a `Request` object and returns a `Response` object.
* @param {string} hook_name
* @param {any} hook_value
*/
export function setHook(hook_name: string, hook_value: any): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly start: () => void;
  readonly version: () => number;
  readonly run: (a: number, b: number) => number;
  readonly setHook: (a: number, b: number, c: number, d: number) => void;
  readonly __main_argc_argv: (a: number, b: number) => number;
  readonly __pink_malloc: (a: number) => number;
  readonly __pink_free: (a: number) => void;
  readonly __pink_realloc: (a: number, b: number) => number;
  readonly __pink_getrandom: (a: number, b: number) => void;
  readonly __pink_clock_time_get: (a: number, b: number, c: number) => number;
  readonly __pink_fd_write: (a: number, b: number, c: number) => number;
  readonly __wbindgen_malloc_command_export: (a: number, b: number) => number;
  readonly __wbindgen_realloc_command_export: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly wasm_bindgen__convert__closures__invoke1_mut__h8c16b6c39c56dff5: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store_command_export: (a: number) => void;
  readonly wasm_bindgen__convert__closures__invoke2_mut__h1ea04ca8ac623e4b: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
