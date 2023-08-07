import { g, connector, config } from '@grafbase/sdk'

const fauna = connector.GraphQL({
  url: 'https://graphql.fauna.com/graphql',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(fauna)

// Use namespaces if you connect multiple APIs to avoid conflicts
// g.datasource(fauna, { namespace: 'Fauna' })

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
