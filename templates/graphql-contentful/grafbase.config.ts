import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

const contentful = connector.GraphQL('Contentful', {
  url: g.env('CONTENTFUL_API_URL'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(contentful)

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
