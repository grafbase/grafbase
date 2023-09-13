import { g, connector, config } from '@grafbase/sdk'

const cosmo = connector.GraphQL('Cosmo', {
  url: g.env('COSMO_API_URL'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
    headers.introspection('Authorization', `Bearer ${g.env('COSMO_API_TOKEN')}`)
  }
})

// Disabling namespace may cause conficts with other connectors
g.datasource(cosmo, { namespace: false })

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
  },
  auth: {
    rules: (rules) => {
      rules.public()
    }
  }
})
