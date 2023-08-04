import { g, connector, config } from '@grafbase/sdk'

const saleor = connector.GraphQL({
  url: g.env('ENVIRONMENT_DOMAIN'),
  headers: (headers) => {
    headers.set('Authorization', `Bearer ${g.env('ACCESS_TOKEN')}`)
  }
})

g.datasource(saleor, { namespace: 'Saleor' })

export default config({
  schema: g
})
