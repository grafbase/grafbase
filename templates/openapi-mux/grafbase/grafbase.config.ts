import { g, connector, config } from '@grafbase/sdk'

const mux = connector.OpenAPI({
  schema: 'https://docs.mux.com/api-spec.json',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(mux)

// Use namespaces if you connect multiple APIs to avoid conflicts
// g.datasource(mux, { namespace: 'Mux' })

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
