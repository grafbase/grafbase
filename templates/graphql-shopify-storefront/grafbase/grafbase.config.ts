import { g, connector, config } from '@grafbase/sdk'

const shopify = connector.GraphQL({
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

// Use namespaces if you connect multiple APIs to avoid conflicts
// g.datasource(shopify, { namespace: 'Shopify' })

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
