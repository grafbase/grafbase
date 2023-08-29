import { g, connector, config } from '@grafbase/sdk'

const mux = connector.OpenAPI('Mux', {
  schema: 'https://docs.mux.com/api-spec.json',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(mux)

// Disabling namespace may cause conficts with other connectors
// g.datasource(mux, { namespace: false })

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
