import { g, connector, config } from '@grafbase/sdk'

const strapi = connector.GraphQL('Strapi', {
  url: g.env('STRAPI_API_URL'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(strapi)

// Disabling namespace may cause conficts with other connectors
// g.datasource(strapi, { namespace: false })

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
