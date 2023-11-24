import * as w from './engine_wasm_stuff/engine_wasm'
import * as fs from 'fs'


function parameterized_query(query: string, params: any[]) {
    console.log("query: " + query + " params: " + params)
    return []
}

function parameterized_execute(query: string, params: any[]) { console.log("execute: " + query + " params: " + params); return 100 }

const config = fs.readFileSync('./registry.json', 'utf8')
const engine = new w.GrafbaseGateway(config, new w.PgCallbacks())

// console.log(JSON.stringify(JSON.parse(await engine.execute(JSON.stringify({ query: "{ __typename __schema { types { name fields { name } } } }" }))), null, 2))

console.log("now querying data")
console.log(JSON.stringify(JSON.parse(await engine.execute(JSON.stringify({ query: "{ pg { testdataCollection(first: 10) { edges { node { id } } } } }" }))), null, 2))
