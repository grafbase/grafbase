import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

const mux = connector.OpenAPI('Mux', {
  schema: 'https://docs.mux.com/api-spec.json',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(mux)

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
