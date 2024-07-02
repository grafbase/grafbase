import { config, graph } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Standalone()

describe('RateLimiting generator', () => {
  beforeEach(() => g.clear())

  it('renders rate limiting with headers', async () => {
    const cfg = config({
      graph: g,
      rateLimiting: {
        rules: [
          {
            name: 'headers',
            limit: 10,
            duration: 10,
            condition: {
              headers: [
                {
                  name: 'my-header',
                  value: '*'
                },
                {
                  name: 'my-header-2',
                  value: ['value1', 'value2']
                }
              ]
            }
          }
        ]
      }
    })

    g.type('A', {
      b: g.int().optional()
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "type A {
        b: Int
      }
      extend schema
        @rateLimiting(rules: [{
            name: "headers",
            limit: 10,
            duration: 10,
            condition: {
              headers: [{name: "my-header", value: "*"},{name: "my-header-2", value: ["value1", "value2"]}]
            }
          }]
        )
      "
    `)
  })

  it('renders rate limiting with jwt claims', async () => {
    const cfg = config({
      graph: g,
      rateLimiting: {
        rules: [
          {
            name: 'jwt_claims',
            limit: 10,
            duration: 10,
            condition: {
              jwtClaims: [
                {
                  name: 'my-claim',
                  value: '*'
                },
                {
                  name: 'my-claim-2',
                  value: 'hello'
                },
                {
                  name: 'my-claim-3',
                  value: { key: 'value' }
                },
                {
                  name: 'my-claim-4',
                  value: ['1', 2]
                }
              ]
            }
          }
        ]
      }
    })

    g.type('A', {
      b: g.int().optional()
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "type A {
        b: Int
      }
      extend schema
        @rateLimiting(rules: [{
            name: "jwt_claims",
            limit: 10,
            duration: 10,
            condition: {
              jwt_claims: [{name: "my-claim", value: "*"},{name: "my-claim-2", value: "hello"},{name: "my-claim-3", value: "{\\"key\\":\\"value\\"}"},{name: "my-claim-4", value: "[\\"1\\",2]"}]
            }
          }]
        )
      "
    `)
  })

  it('renders rate limiting with ips', async () => {
    const cfg = config({
      graph: g,
      rateLimiting: {
        rules: [
          {
            name: 'all_ips',
            limit: 10,
            duration: 10,
            condition: {
              ips: '*'
            }
          },
          {
            name: 'specific_ips',
            limit: 10,
            duration: 10,
            condition: {
              ips: ['1.1.1.1']
            }
          }
        ]
      }
    })

    g.type('A', {
      b: g.int().optional()
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "type A {
        b: Int
      }
      extend schema
        @rateLimiting(rules: [{
            name: "all_ips",
            limit: 10,
            duration: 10,
            condition: {
              ips: "*"
            }
          },{
            name: "specific_ips",
            limit: 10,
            duration: 10,
            condition: {
              ips: ["1.1.1.1"]
            }
          }]
        )
      "
    `)
  })

  it('renders rate limiting with operations', async () => {
    const cfg = config({
      graph: g,
      rateLimiting: {
        rules: [
          {
            name: 'all_operations',
            limit: 10,
            duration: 10,
            condition: {
              operations: '*'
            }
          },
          {
            name: 'specific_operations',
            limit: 10,
            duration: 10,
            condition: {
              operations: ['hello']
            }
          }
        ]
      }
    })

    g.type('A', {
      b: g.int().optional()
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "type A {
        b: Int
      }
      extend schema
        @rateLimiting(rules: [{
            name: "all_operations",
            limit: 10,
            duration: 10,
            condition: {
              operations: "*"
            }
          },{
            name: "specific_operations",
            limit: 10,
            duration: 10,
            condition: {
              operations: ["hello"]
            }
          }]
        )
      "
    `)
  })
})
