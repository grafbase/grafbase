import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

const neon = connector.OpenAPI('Neon', {
  schema: 'https://console.neon.tech/api/v2/',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(neon)

export default config({
  graph: g,
  cache: {
    rules: [
      {
        types: ['Query'],
        maxAge: 60
      }
    ]
  },
  auth: {
    rules: (rules) => {
      rules.public()
    }
  }
})
