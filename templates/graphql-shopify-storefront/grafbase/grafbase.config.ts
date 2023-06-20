import { g, connector, config } from '@grafbase/sdk'

const shopify = connector.GraphQL({
  url: `https://${g.env(
    'SHOPIFY_STORE_NAME'
  )}.myshopify.com/api/2023-04/graphql.json`,
  headers: (headers) => {
    headers.static(
      'X-Shopify-Storefront-Access-Token',
      g.env('SHOPIFY_STOREFRONT_ACCESS_TOKEN')
    )
  }
})

g.datasource(shopify, { namespace: 'Shopify' })

export default config({
  schema: g
})
