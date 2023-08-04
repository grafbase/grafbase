import { g, connector, config } from '@grafbase/sdk'

const tinybird = connector.OpenAPI({
  schema: g.env('TINYBIRD_API_SCHEMA'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(tinybird)

// Use namespaces if you connect multiple APIs to avoid conflicts
// g.datasource(tinybird, { namespace: 'Tinybird' })

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
