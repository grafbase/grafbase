import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone({ subgraph: true })

const openapi = connector.OpenAPI('OpenAPI', {
  schema: g.env('SCHEMA_URL')
})

g.datasource(openapi, { namespace: false })

export default config({
  graph: g,
  auth: {
    rules: (rules) => {
      rules.public()
    }
  }
})
