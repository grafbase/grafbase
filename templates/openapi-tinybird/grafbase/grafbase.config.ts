import { g, connector, config } from '@grafbase/sdk'

const tinybird = connector.OpenAPI({
  schema: g.env('TINYBIRD_API_SCHEMA'),
  headers: (headers) => {
    headers.static('Authorization', `Bearer ${g.env('TINYBIRD_API_TOKEN')}`)
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(tinybird, { namespace: 'Tinybird' })

export default config({
  schema: g
})
