import { g, connector, config } from '@grafbase/sdk'

const dgraph = connector.GraphQL('Dgraph', {
  url: g.env('DGRAPH_API_URL')
})

g.datasource(dgraph)

// Disabling namespace may cause conficts with other connectors
// g.datasource(dgraph, { namespace: false })

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
