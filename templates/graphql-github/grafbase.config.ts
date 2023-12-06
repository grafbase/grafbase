import { g, connector, config } from '@grafbase/sdk'

const github = connector.GraphQL('GitHub', {
  url: 'https://api.github.com/graphql',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(github)

// Disabling namespace may cause conficts with other connectors
// g.datasource(github, { namespace: false })

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
