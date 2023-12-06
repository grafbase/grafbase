import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

const datocms = connector.GraphQL('DatoCMS', {
  url: 'https://graphql.datocms.com',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(datocms)

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
