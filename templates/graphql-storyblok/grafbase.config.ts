import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

const storyblok = connector.GraphQL('Storyblok', {
  url: 'https://gapi.storyblok.com/v1/api',
  headers: (headers) => {
    headers.set('Token', { forward: 'Token' })
    headers.set('Version', { forward: 'Version' })
  }
})

g.datasource(storyblok)

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
