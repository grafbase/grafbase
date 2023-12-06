import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

const cosmo = connector.GraphQL('Cosmo', {
  url: g.env('COSMO_API_URL'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
    headers.introspection('Authorization', `Bearer ${g.env('COSMO_API_TOKEN')}`)
  }
})

g.datasource(cosmo)

export default config({
  graph: g,
  cache: {
    rules: [
      {
        types: ['Query'],
        maxAge: 60,
        staleWhileRevalidate: 60
      }
    ]
  },
  auth: {
    rules: (rules) => {
      rules.public()
    }
  }
})
