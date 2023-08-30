import { g, connector, config } from '@grafbase/sdk'

const fauna = connector.GraphQL('Fauna', {
  url: 'https://graphql.fauna.com/graphql',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(fauna)

// Disabling namespace may cause conficts with other connectors
// g.datasource(fauna, { namespace: false })

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
