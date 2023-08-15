import { g, connector, config } from '@grafbase/sdk'

const github = connector.OpenAPI({
  schema:
    'https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/ghes-3.0/ghes-3.0.json',
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
