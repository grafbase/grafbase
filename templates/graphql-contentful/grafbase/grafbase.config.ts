import { g, connector, config } from '@grafbase/sdk'

const contentful = connector.GraphQL('Contentful', {
  url: g.env('CONTENTFUL_API_URL'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(contentful)

// Disabling namespace may cause conficts with other connectors
// g.datasource(contentful, { namespace: false })

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
