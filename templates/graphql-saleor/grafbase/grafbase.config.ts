import { g, connector, config } from '@grafbase/sdk'

const saleor = connector.GraphQL({
  url: g.env('ENVIRONMENT_DOMAIN'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(saleor)

// Use namespaces if you connect multiple APIs to avoid conflicts
// g.datasource(saleor, { namespace: 'Saleor' })

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
