import { g, connector, config } from '@grafbase/sdk'

const openai = connector.OpenAPI('OpenAI', {
  schema:
    'https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(openai)

// Disabling namespace may cause conficts with other connectors
// g.datasource(openai, { namespace: false })

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
