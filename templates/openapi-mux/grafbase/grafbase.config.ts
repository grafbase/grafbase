import { g, connector, config } from '@grafbase/sdk'

const mux = connector.OpenAPI({
  schema: 'https://docs.mux.com/api-spec.json',
  headers: (headers) => {
    headers.set('Authorization', `Basic ${g.env('MUX_BASE64')}`)
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(mux, { namespace: 'Mux' })

export default config({
  schema: g
})
