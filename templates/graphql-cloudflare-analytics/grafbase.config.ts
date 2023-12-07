import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

const cloudflare = connector.GraphQL('Cloudflare', {
  url: 'https://api.cloudflare.com/client/v4/graphql',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(cloudflare)

export default config({
  graph: g,
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
