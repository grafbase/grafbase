import * as w from './engine_wasm_stuff/engine_wasm'
import * as fs from 'fs'

import { Client } from 'pg'
const client = new Client(process.env.POSTGRES_URL)
await client.connect()

async function parameterized_query(query: string, params: any[]) {
    console.log("query: " + query + " params: " + params)
    let result = await client.query(query, params)
    return result.rows[0]
}

async function parameterized_execute(query: string, params: any[]) {
    console.log("execute: " + query + " params: " + params);
    let result = await client.execute(query, params)
    console.log("result: " + JSON.stringify(result))
    return 100
}

const config = fs.readFileSync('./registry.json', 'utf8')
const engine = new w.GrafbaseGateway(config, new w.PgCallbacks(parameterized_execute, parameterized_query))

// console.log(JSON.stringify(JSON.parse(await engine.execute(JSON.stringify({ query: "{ __typename __schema { types { name fields { name } } } }" }))), null, 2))

console.log("now querying data")
console.log(JSON.stringify(JSON.parse(await engine.execute(JSON.stringify({ query: "{ pg { testdataCollection(first: 10) { edges { node { id name } } } } }" }))), null, 2))

console.log(JSON.stringify(JSON.parse(await engine.execute(JSON.stringify({ query: "mutation { pg { testdataDelete(by: { id: 1 }) { rowCount } } }" }))), null, 2))


console.log(JSON.stringify(JSON.parse(await engine.execute(JSON.stringify({ query: "{ swapi { allFilms(first: 3) { films { title director } } } }" }))), null, 2))

console.log(JSON.stringify(JSON.parse(await engine.execute(JSON.stringify({ query: "{ openapi { account { country } } }" }))), null, 2))
