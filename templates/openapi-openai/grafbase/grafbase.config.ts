import { g, connector, config } from '@grafbase/sdk'

const openai = connector.OpenAPI({
  schema:
    'https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(openai)

// Use namespaces if you connect multiple APIs to avoid conflicts
// g.datasource(openai, { namespace: 'OpenAI' })

export default config({
  schema: g,
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
