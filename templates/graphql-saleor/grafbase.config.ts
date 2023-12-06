import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

const saleor = connector.GraphQL('Saleor', {
  url: g.env('ENVIRONMENT_DOMAIN'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(saleor)

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
