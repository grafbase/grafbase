import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

const fauna = connector.GraphQL('Fauna', {
  url: 'https://graphql.fauna.com/graphql',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(fauna)

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
