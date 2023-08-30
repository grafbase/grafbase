import { g, connector, config } from '@grafbase/sdk'

const stripe = connector.OpenAPI('Stripe', {
  schema:
    'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(stripe)

// Disabling namespace may cause conficts with other connectors
// g.datasource(stripe, { namespace: false })

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
