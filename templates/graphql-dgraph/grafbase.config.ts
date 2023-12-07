import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

const dgraph = connector.GraphQL('Dgraph', {
  url: g.env('DGRAPH_API_URL')
})

g.datasource(dgraph)

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
