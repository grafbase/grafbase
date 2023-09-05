import { g, connector, config } from '@grafbase/sdk'

const datocms = connector.GraphQL('DatoCMS', {
  url: 'https://graphql.datocms.com',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

// Disabling namespace may cause conflicts with other connectors
g.datasource(datocms, { namespace: false })

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
