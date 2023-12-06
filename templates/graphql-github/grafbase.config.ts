import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

const github = connector.GraphQL('GitHub', {
  url: 'https://api.github.com/graphql',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(github)

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
