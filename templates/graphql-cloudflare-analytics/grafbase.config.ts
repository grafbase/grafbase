import { g, connector, config } from '@grafbase/sdk'

const cloudflare = connector.GraphQL('Cloudflare', {
  url: 'https://api.cloudflare.com/client/v4/graphql',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(cloudflare)

// Disabling namespace may cause conficts with other connectors
// g.datasource(cloudflare, { namespace: false })

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
