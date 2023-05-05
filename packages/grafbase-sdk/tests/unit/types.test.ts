import { g, config } from '../../src/index'
import { describe, expect, it } from '@jest/globals'

describe('Type generator', () => {
  it('generates one with a single field', () => {
    const t = g.type('User', {
      name: g.string()
    })

    expect(t.toString()).toMatchInlineSnapshot(`
      "type User {
        name: String!
      }"
    `)
  })

  it('generates one with many fields', () => {
    const t = g.type('User', {
      name: g.string(),
      age: g.int().optional()
    })

    expect(t.toString()).toMatchInlineSnapshot(`
      "type User {
        name: String!
        age: Int
      }"
    `)
  })

  it('generates a union of multiple types', () => {
    const user = g.type('User', {
      name: g.string(),
      age: g.int().optional()
    })

    const address = g.type('Address', {
      street: g.string().optional()
    })

    const union = g.union('UserOrAddress', { user, address })

    const cfg = config().schema({
      types: [user, address],
      unions: [union]
    })

    expect(cfg.toString()).toMatchInlineSnapshot(`
      "type User {
        name: String!
        age: Int
      }

      type Address {
        street: String
      }

      union UserOrAddress = User | Address"
    `)
  })
})
