import { g, connector, config } from '@grafbase/sdk'

const neon = connector.OpenAPI({
  schema: 'https://console.neon.tech/api/v2/',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(neon)

// Use namespaces if you connect multiple APIs to avoid conflicts
// g.datasource(neon, { namespace: 'Neon' })

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
