import { g, config } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

describe('Type generator', () => {
  beforeEach(() => {
    g.clear()
  })

  it('generates one with a single field', () => {
    const t = g.type('User', {
      name: g.string()
    })

    expect(renderGraphQL(t)).toMatchInlineSnapshot(`
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

    expect(renderGraphQL(t)).toMatchInlineSnapshot(`
      "type User {
        name: String!
        age: Int
      }"
    `)
  })

  it('generates one with cache', () => {
    const t = g
      .type('User', {
        name: g.string().cache({ maxAge: 10, staleWhileRevalidate: 20 })
      })
      .cache({ maxAge: 10, staleWhileRevalidate: 20 })

    expect(renderGraphQL(t)).toMatchInlineSnapshot(`
      "type User @cache(maxAge: 10, staleWhileRevalidate: 20) {
        name: String! @cache(maxAge: 10, staleWhileRevalidate: 20)
      }"
    `)
  })

  it('generates one with cache using type mutation invalidation', () => {
    const t = g
      .type('User', {
        name: g.string()
      })
      .cache({ maxAge: 10, mutationInvalidation: 'type' })

    expect(renderGraphQL(t)).toMatchInlineSnapshot(`
      "type User @cache(maxAge: 10, mutationInvalidation: type) {
        name: String!
      }"
    `)
  })

  it('generates one with cache using entity mutation invalidation', () => {
    const t = g
      .type('User', {
        name: g.string()
      })
      .cache({ maxAge: 10, mutationInvalidation: 'entity' })

    expect(renderGraphQL(t)).toMatchInlineSnapshot(`
      "type User @cache(maxAge: 10, mutationInvalidation: entity) {
        name: String!
      }"
    `)
  })

  it('generates one with cache using list mutation invalidation', () => {
    const t = g
      .type('User', {
        name: g.string()
      })
      .cache({ maxAge: 10, mutationInvalidation: 'list' })

    expect(renderGraphQL(t)).toMatchInlineSnapshot(`
      "type User @cache(maxAge: 10, mutationInvalidation: list) {
        name: String!
      }"
    `)
  })

  it('generates one with cache using custom field mutation invalidation', () => {
    const t = g
      .type('User', {
        name: g.string()
      })
      .cache({ maxAge: 10, mutationInvalidation: { field: 'name' } })

    expect(renderGraphQL(t)).toMatchInlineSnapshot(`
      "type User @cache(maxAge: 10, mutationInvalidation: { field: "name" }) {
        name: String!
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

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
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

  it('references another type', () => {
    g.type('User', {
      name: g.string(),
      age: g.int().optional()
    })

    const city = g.type('City', {
      country: g.string()
    })

    g.type('Address', {
      street: g.string().optional(),
      city: g.ref(city)
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User {
        name: String!
        age: Int
      }

      type City {
        country: String!
      }

      type Address {
        street: String
        city: City!
      }"
    `)
  })

  it('references another an enum', () => {
    g.type('User', {
      name: g.string(),
      age: g.int().optional()
    })

    const enm = g.enum('Color', ['Red', 'Green'])

    g.type('Address', {
      street: g.string().optional(),
      color: g.enumRef(enm).optional()
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "enum Color {
        Red,
        Green
      }

      type User {
        name: String!
        age: Int
      }

      type Address {
        street: String
        color: Color
      }"
    `)
  })
})
