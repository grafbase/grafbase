import { g, connector, config } from '@grafbase/sdk'

const cloudflare = connector.GraphQL({
  url: 'https://api.cloudflare.com/client/v4/graphql',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(cloudflare)

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
