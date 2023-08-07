import { g, connector, config } from '@grafbase/sdk'

const github = connector.GraphQL({
  url: 'https://api.github.com/graphql',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(github)

// Use namespaces if you connect multiple APIs to avoid conflicts
// g.datasource(github, { namespace: 'GitHub' })

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
