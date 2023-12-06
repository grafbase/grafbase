import { g, connector, config } from '@grafbase/sdk'

const commercetools = connector.GraphQL('commercetools', {
  url: g.env('COMMERCETOOLS_API_URL'),
  headers: (headers) => {
    headers.set('Authorization', { forward: 'Authorization' })
    headers.introspection(
      'Authorization',
      `Bearer ${g.env('COMMERCETOOLS_API_TOKEN')}`
    )
  }
})

// Disabling namespace may cause conficts with other connectors
g.datasource(commercetools, { namespace: false })

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
