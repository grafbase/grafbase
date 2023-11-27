/* tslint:disable */
/* eslint-disable */
/**
* Configuration options for Cloudflare's image optimization feature:
* <https://blog.cloudflare.com/introducing-polish-automatic-image-optimizati/>
*/
export enum PolishConfig {
  Off = 0,
  Lossy = 1,
  Lossless = 2,
}
/**
*/
export enum RequestRedirect {
  Error = 0,
  Follow = 1,
  Manual = 2,
}
/**
*/
export class GrafbaseGateway {
  free(): void;
/**
* @param {string} schema
* @param {PgCallbacks | undefined} pg_callbacks
*/
  constructor(schema: string, pg_callbacks?: PgCallbacks);
/**
* @param {string} request
* @returns {Promise<string>}
*/
  execute(request: string): Promise<string>;
}
/**
*/
export class IntoUnderlyingByteSource {
  free(): void;
/**
* @param {any} controller
*/
  start(controller: any): void;
/**
* @param {any} controller
* @returns {Promise<any>}
*/
  pull(controller: any): Promise<any>;
/**
*/
  cancel(): void;
/**
*/
  readonly autoAllocateChunkSize: number;
/**
*/
  readonly type: string;
}
/**
*/
export class IntoUnderlyingSink {
  free(): void;
/**
* @param {any} chunk
* @returns {Promise<any>}
*/
  write(chunk: any): Promise<any>;
/**
* @returns {Promise<any>}
*/
  close(): Promise<any>;
/**
* @param {any} reason
* @returns {Promise<any>}
*/
  abort(reason: any): Promise<any>;
}
/**
*/
export class IntoUnderlyingSource {
  free(): void;
/**
* @param {any} controller
* @returns {Promise<any>}
*/
  pull(controller: any): Promise<any>;
/**
*/
  cancel(): void;
}
/**
* Configuration options for Cloudflare's minification features:
* <https://www.cloudflare.com/website-optimization/>
*/
export class MinifyConfig {
  free(): void;
/**
*/
  css: boolean;
/**
*/
  html: boolean;
/**
*/
  js: boolean;
}
/**
*/
export class PgCallbacks {
  free(): void;
/**
* @param {Function} parameterized_execute
* @param {Function} parameterized_query
*/
  constructor(parameterized_execute: Function, parameterized_query: Function);
}
/**
* Raw options for [`pipeTo()`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/pipeTo).
*/
export class PipeOptions {
  free(): void;
/**
*/
  readonly preventAbort: boolean;
/**
*/
  readonly preventCancel: boolean;
/**
*/
  readonly preventClose: boolean;
/**
*/
  readonly signal: AbortSignal | undefined;
}
/**
*/
export class QueuingStrategy {
  free(): void;
/**
*/
  readonly highWaterMark: number;
}
/**
*/
export class R2Range {
  free(): void;
/**
*/
  length?: number;
/**
*/
  offset?: number;
/**
*/
  suffix?: number;
}
/**
* Raw options for [`getReader()`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/getReader).
*/
export class ReadableStreamGetReaderOptions {
  free(): void;
/**
*/
  readonly mode: any;
}
