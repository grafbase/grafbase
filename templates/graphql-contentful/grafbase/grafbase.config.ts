import { g, connector, config } from '@grafbase/sdk'

const contentful = connector.GraphQL({
  url: g.env('CONTENTFUL_API_URL'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(contentful)

// Use namespaces if you connect multiple APIs to avoid conflicts
// g.datasource(contentful, { namespace: 'Contentful' })

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
