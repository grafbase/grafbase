import { g, connector, config } from '@grafbase/sdk'

const strapi = connector.GraphQL({
  url: g.env('STRAPI_API_URL'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(strapi)

// Use namespaces if you connect multiple APIs to avoid conflicts
// g.datasource(strapi, { namespace: 'Strapi' })

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
