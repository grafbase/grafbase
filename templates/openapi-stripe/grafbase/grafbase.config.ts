import { g, connector, config } from '@grafbase/sdk'

const stripe = connector.OpenAPI({
  schema:
    'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(stripe)

// Use namespaces if you connect multiple APIs to avoid conflicts
// g.datasource(stripe, { namespace: 'Stripe' })

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
