import { g, connector, config } from '@grafbase/sdk'

const github = connector.OpenAPI({
  schema:
    'https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/ghes-3.0/ghes-3.0.json',
  headers: (headers) => {
    headers.static('Authorization', `Bearer ${g.env('GITHUB_ACCESS_TOKEN')}`)
  }
})

g.datasource(github, { namespace: 'GitHub' })

export default config({
  schema: g
})
