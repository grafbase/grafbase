import { g, connector, config } from '@grafbase/sdk'

const storyblok = connector.GraphQL('Storyblok', {
  url: 'https://gapi.storyblok.com/v1/api',
  headers: (headers) => {
    headers.set('Token', { forward: 'Token' })
    headers.set('Version', { forward: 'Version' })
  }
})

g.datasource(storyblok)

// Disabling namespace may cause conficts with other connectors
// g.datasource(storyblok, { namespace: false })

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
