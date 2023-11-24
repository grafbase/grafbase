import * as w from './engine_wasm_stuff/engine_wasm'
import * as fs from 'fs'

const config = fs.readFileSync('./registry.json', 'utf8')
const engine = new w.GrafbaseGateway(config)

console.log(await engine.execute(JSON.stringify({ query: "{ __typename }" })))
