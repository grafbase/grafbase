import { config, graph, auth } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Single()

describe('Authorizer auth provider', () => {
  beforeEach(() => g.clear())

  it('renders a provider with private access', () => {
    const narf = auth.Authorizer({
      name: 'narf'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [narf],
        rules: (rules) => {
          rules.private()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: authorizer, name: "narf" }
          ]
          rules: [
            { allow: private }
          ]
        )"
    `)
  })
})
