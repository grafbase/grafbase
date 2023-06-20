import { g, connector, config } from '@grafbase/sdk'

const contentful = connector.GraphQL({
  url: g.env('CONTENTFUL_API_URL'),
  headers: (headers) => {
    headers.static('Authorization', `Bearer ${g.env('CONTENTFUL_API_TOKEN')}`)
  }
})

g.datasource(contentful, { namespace: 'Contentful' })

export default config({
  schema: g
})
