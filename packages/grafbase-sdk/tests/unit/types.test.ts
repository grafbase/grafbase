import { g, config } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'

describe('Type generator', () => {
  beforeEach(() => {
    g.clear()  
  })

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

    g.union('UserOrAddress', { user, address })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
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
