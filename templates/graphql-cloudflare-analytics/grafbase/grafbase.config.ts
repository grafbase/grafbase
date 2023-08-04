import { g, connector, config } from '@grafbase/sdk'

const cloudflare = connector.GraphQL({
  url: 'https://api.cloudflare.com/client/v4/graphql',
  headers: (headers) => {
    headers.set('Authorization', `Bearer ${g.env('API_TOKEN')}`)
  }
})

g.datasource(cloudflare, { namespace: 'Cloudflare' })

export default config({
  schema: g
})
