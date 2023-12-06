import { g, connector, config } from '@grafbase/sdk'

const neon = connector.OpenAPI('Neon', {
  schema: 'https://console.neon.tech/api/v2/',
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(neon)

// Disabling namespace may cause conficts with other connectors
// g.datasource(neon, { namespace: false })

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
