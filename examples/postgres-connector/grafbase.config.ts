import { g, config, connector } from '@grafbase/sdk'

// Welcome to Grafbase!
//
// Configure authentication, data sources, resolvers and caching for your GraphQL API.

// Data Sources - https://grafbase.com/docs/connectors

const pg = connector.Postgres('pg', { url: g.env('POSTGRES_URL') })
g.datasource(pg)

const gql = connector.GraphQL('swapi', { url: 'https://swapi-graphql.netlify.app/.netlify/functions/index' })
g.datasource(gql)

const stripe = connector.OpenAPI('Stripe', {
  schema:
    'https://api.apis.guru/v2/specs/openapi.space/1.0.0/swagger.json',
  // headers: headers => {
  //   headers.set('Authorization', `Bearer ${g.env('STRIPE_API_KEY')}`)
  // },
})

g.datasource(stripe)

// Resolvers - https://grafbase.com/docs/resolvers
//
// g.query('helloWorld', {
//   returns: g.string(),
//   resolver: 'hello-world',
// })

export default config({
  schema: g,
  // Authentication - https://grafbase.com/docs/auth
  auth: {
    // OpenID Connect
    // const oidc = auth.OpenIDConnect({ issuer: g.env('OIDC_ISSUER_URL') })
    // providers: [oidc],
    rules: (rules) => {
      rules.public()
    },
  },
  // Caching - https://grafbase.com/docs/graphql-edge-caching
  // cache: {
  //   rules: [
  //     {
  //       types: ['Query'], // Cache everything for 60 seconds
  //       maxAge: 60,
  //       staleWhileRevalidate: 60
  //     }
  //   ]
  // }
})
