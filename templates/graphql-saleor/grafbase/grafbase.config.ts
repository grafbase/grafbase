import { g, connector, config } from '@grafbase/sdk'

const saleor = connector.GraphQL('Saleor', {
  url: g.env('ENVIRONMENT_DOMAIN'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(saleor)

// Disabling namespace may cause conficts with other connectors
// g.datasource(saleor, { namespace: false })

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
