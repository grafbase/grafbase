import { g, connector, config } from '@grafbase/sdk'

const github = connector.GraphQL({
  url: 'https://api.github.com/graphql',
  headers: (headers) => {
    headers.set('Authorization', `Bearer ${g.env('GITHUB_ACCESS_TOKEN')}`)
  }
})

g.datasource(github, { namespace: 'GitHub' })

export default config({
  schema: g
})
