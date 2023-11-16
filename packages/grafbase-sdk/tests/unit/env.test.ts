import { graph, connector, config } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'

const g = graph.Single()

describe('Env var accessor', () => {
  beforeEach(() => g.clear())

  it('returns the value of the variable if set', () => {
    process.env.TEST_VAL = 'test'

    expect(g.env('TEST_VAL')).toBe('test')

    delete process.env.TEST_VAL
  })

  it('throws if the variable is not set', () => {
    expect(() => g.env('TEST_VAL')).toThrow(
      'Environment variable TEST_VAL is not set'
    )
  })

  it('adds the variable to the SDL', () => {
    process.env.GITHUB_TOKEN = 'test_token'

    const github = connector.GraphQL('GitHub', {
      url: 'https://api.github.com/graphql',
      headers: (headers) => {
        headers.static('Authorization', `Bearer ${g.env('GITHUB_TOKEN')}`)
      }
    })

    g.datasource(github)

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "extend schema
        @graphql(
          name: "GitHub"
          namespace: true
          url: "https://api.github.com/graphql"
          headers: [
            { name: "Authorization", value: "Bearer test_token" }
          ]
        )"
    `)

    delete process.env.GITHUB_TOKEN
  })
})
