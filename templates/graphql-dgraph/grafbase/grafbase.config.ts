import { g, connector, config } from '@grafbase/sdk'

const dgraph = connector.GraphQL({
  url: g.env('DGRAPH_API_URL')

  // Enable headers if your Dgraph Cloud API requires it
  // headers: (headers) => {
  //   headers.set('Authorization', { forward: 'Authorization' })
  // }
})

g.datasource(dgraph)

// Use namespaces if you connect multiple APIs to avoid conflicts
// g.datasource(dgraph, { namespace: 'Dgraph' })

export default config({
  schema: g,
  cache: {
    rules: [
      {
        types: ['Query'],
        maxAge: 60,
        staleWhileRevalidate: 60
      }
    ]
  }
})
