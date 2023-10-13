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

  it('generates one with cache using access scopes', () => {
    const t = g
      .type('User', {
        name: g.string().cache({
          maxAge: 10,
          staleWhileRevalidate: 20,
          scopes: ['apikey', { header: 'test' }, 'public']
        })
      })
      .cache({
        maxAge: 10,
        staleWhileRevalidate: 20,
        scopes: [{ claim: 'test' }]
      })

    expect(renderGraphQL(t)).toMatchInlineSnapshot(`
      "type User @cache(maxAge: 10, staleWhileRevalidate: 20, scopes: [{ claim: "test" }]) {
        name: String! @cache(maxAge: 10, staleWhileRevalidate: 20, scopes: [apikey, { header: "test" }, public])
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

  it('prevents using of whitespaced identifier as a union name', () => {
    const user = g.type('User', {
      name: g.string(),
      age: g.int().optional()
    })

    const address = g.type('Address', {
      street: g.string().optional()
    })

    expect(() => g.union('white space', { user, address })).toThrow(
      'Given name "white space" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of number-prefixed identifier as a union name', () => {
    const user = g.type('User', {
      name: g.string(),
      age: g.int().optional()
    })

    const address = g.type('Address', {
      street: g.string().optional()
    })

    expect(() => g.union('0User', { user, address })).toThrow(
      'Given name "0User" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of weird characters identifier as a union name', () => {
    const user = g.type('User', {
      name: g.string(),
      age: g.int().optional()
    })

    const address = g.type('Address', {
      street: g.string().optional()
    })

    expect(() => g.union('!@#$%^&*()+|~`=-', { user, address })).toThrow(
      'Given name "!@#$%^&*()+|~`=-" is not a valid TypeScript identifier.'
    )
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

  it('prevents using of whitespaced identifier as the name', () => {
    expect(() => g.type('white space', { name: g.string() })).toThrow(
      'Given name "white space" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of number-prefixed identifier as the name', () => {
    expect(() => g.type('0User', { name: g.string() })).toThrow(
      'Given name "0User" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of weird characters identifier as the name', () => {
    expect(() => g.type('!@#$%^&*()+|~`=-', { name: g.string() })).toThrow(
      'Given name "!@#$%^&*()+|~`=-" is not a valid TypeScript identifier.'
    )
  })

  it('extends an internal type', () => {
    const t = g.type('User', {
      name: g.string()
    })

    g.extend(t, {
      myField: {
        args: { myArg: g.string() },
        returns: g.string(),
        resolver: 'file'
      }
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User {
        name: String!
      }

      extend type User {
        myField(myArg: String!): String! @resolver(name: "file")
      }"
    `)
  })

  it('extends an external type', () => {
    g.extend('StripeCustomer', {
      myField: {
        args: { myArg: g.string() },
        returns: g.string(),
        resolver: 'file'
      }
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend type StripeCustomer {
        myField(myArg: String!): String! @resolver(name: "file")
      }"
    `)
  })

  it('supports field resolvers', () => {
    g.type('User', {
      name: g.string().resolver('a-field')
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User {
        name: String! @resolver(name: "a-field")
      }"
    `)
  })

  it('supports federation keys', () => {
    g.type('User', {
      id: g.id()
    }).key('id')

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
"type User @key(fields: "id" resolvable: true) {
  id: ID!
}"
`)
  })

  it('supports unresolvable federation keys', () => {
    g.type('User', {
      id: g.id()
    }).key('id', { resolvable: false })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
"type User @key(fields: "id" resolvable: false) {
  id: ID!
}"
`)
  })
})
