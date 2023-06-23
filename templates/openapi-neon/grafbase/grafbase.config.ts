import { g, connector, config } from '@grafbase/sdk'

const neon = connector.OpenAPI({
  schema: 'https://console.neon.tech/api/v2/',
  headers: (headers) => {
    headers.static('Authorization', `Bearer ${g.env('NEON_API_KEY')}`)
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(neon, { namespace: 'Neon' })

export default config({
  schema: g
})
