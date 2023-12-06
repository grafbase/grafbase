import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

const strapi = connector.GraphQL('Strapi', {
  url: g.env('STRAPI_API_URL'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(strapi)

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
