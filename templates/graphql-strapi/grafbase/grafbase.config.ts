import { g, connector, config } from '@grafbase/sdk'

const contentful = connector.GraphQL({
  url: g.env('STRAPI_API_URL'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
  }
})

g.datasource(contentful, { namespace: 'Strapi' })

export default config({
  schema: g,
  cache: {
    rules: [
      {
        types: ['StrapiQuery'],
        maxAge: 60,
        mutationInvalidation: 'entity'
      }
    ]
  }
})
