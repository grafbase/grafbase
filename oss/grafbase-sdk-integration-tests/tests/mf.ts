import { Log, LogLevel, Miniflare, Response } from 'miniflare'

export const MF_RUST_WORKER_MISSING_BASE_PREFIX = new Miniflare({
  scriptPath: './build/worker/shim.mjs',
  name: 'rust-grafbase-sdk',
  compatibilityDate: '2023-05-18',
  kvPersist: false,
  log: new Log(LogLevel.DEBUG),
  modules: true,
  modulesRules: [{ type: 'CompiledWasm', include: ['**/*.wasm'] }],
  kvNamespaces: ['TEST_NAMESPACE'],
  bindings: {
    // needs to be mounted through kvNamespaces
    KV_ID: 'TEST_NAMESPACE',
  },
})

export const MF_RUST_WORKER = new Miniflare({
  scriptPath: './build/worker/shim.mjs',
  name: 'rust-grafbase-sdk',
  compatibilityDate: '2023-05-18',
  kvPersist: false,
  log: new Log(LogLevel.DEBUG),
  modules: true,
  modulesRules: [{ type: 'CompiledWasm', include: ['**/*.wasm'] }],
  kvNamespaces: ['TEST_NAMESPACE'],
  bindings: {
    KV_BASE_PREFIX: 'test',
    // needs to be mounted through kvNamespaces
    KV_ID: 'TEST_NAMESPACE',
  },
})

export const MF_JS_WORKER = new Miniflare({
  scriptPath: './js/dist/worker.js',
  name: 'js-grafbase-sdk',
  compatibilityDate: '2023-05-18',
  kvPersist: false,
  log: new Log(LogLevel.DEBUG),
  modules: true,
  modulesRules: [{ type: 'CompiledWasm', include: ['**/*.wasm'] }],
  kvNamespaces: ['TEST_NAMESPACE'],
  bindings: {
    KV_BASE_PREFIX: 'test',
    // needs to be mounted through kvNamespaces
    KV_ID: 'TEST_NAMESPACE',
  },
})
