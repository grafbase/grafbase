import init from '../pkg'
import wasmData from '../pkg/index_bg.wasm'

await init(wasmData)

export * from '../pkg'
