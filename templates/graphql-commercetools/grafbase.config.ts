import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

const commercetools = connector.GraphQL('commercetools', {
  url: g.env('COMMERCETOOLS_API_URL'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
    headers.introspection(
      'Authorization',
      `Bearer ${g.env('COMMERCETOOLS_API_TOKEN')}`
    )
  }
})

g.datasource(commercetools)

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
