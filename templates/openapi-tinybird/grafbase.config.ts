import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

const tinybird = connector.OpenAPI('Tinybird', {
  schema: g.env('TINYBIRD_API_SCHEMA'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(tinybird)

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
