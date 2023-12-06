import { g, connector, config } from '@grafbase/sdk'

const shopify = connector.GraphQL('Shopify', {
  url: `https://${g.env(
    'SHOPIFY_STORE_NAME'
  )}.myshopify.com/api/2023-04/graphql.json`,
  headers: (headers) => {
    headers.set('X-Shopify-Storefront-Access-Token', {
      forward: 'X-Shopify-Storefront-Access-Token'
    })
  }
})

g.datasource(shopify)

// Disabling namespace may cause conficts with other connectors
// g.datasource(shopify, { namespace: false })

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
