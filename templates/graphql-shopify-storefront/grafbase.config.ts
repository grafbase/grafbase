import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

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

export default config({
  graph: g,
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
