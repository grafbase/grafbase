import { g, connector, config } from '@grafbase/sdk'

const tinybird = connector.OpenAPI('Tinybird', {
  schema: g.env('TINYBIRD_API_SCHEMA'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(tinybird)

// Disabling namespace may cause conficts with other connectors
// g.datasource(tinybird, { namespace: false })

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
