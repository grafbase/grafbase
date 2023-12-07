import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

const openai = connector.OpenAPI('OpenAI', {
  schema:
    'https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(openai)

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
