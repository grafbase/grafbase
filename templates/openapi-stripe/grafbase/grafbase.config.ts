import { g, connector, config } from '@grafbase/sdk'

const stripe = connector.OpenAPI({
  schema:
    'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json',
  headers: (headers) => {
    headers.set('Authorization', `Bearer ${g.env('STRIPE_API_KEY')}`)
  },
  transforms: { queryNaming: 'OPERATION_ID' }
})

g.datasource(stripe, { namespace: 'Stripe' })

export default config({
  schema: g
})
