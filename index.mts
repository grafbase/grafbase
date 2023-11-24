import * as w from './engine_wasm_stuff/engine_wasm'
import * as fs from 'fs'

const config = fs.readFileSync('./registry.json', 'utf8')
const engine = new w.GrafbaseGateway(config)

// console.log(JSON.stringify(JSON.parse(await engine.execute(JSON.stringify({ query: "{ __typename __schema { types { name fields { name } } } }" }))), null, 2))

console.log("now querying data")
console.log(JSON.stringify(JSON.parse(await engine.execute(JSON.stringify({ query: "{ pg { testdataCollection { edges { node { id } } } } }" }))), null, 2))
