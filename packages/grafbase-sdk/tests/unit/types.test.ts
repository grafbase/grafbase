import { graph, config } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Single()

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

  it('supports field resolvers with requires directive', () => {
    g.type('User', {
      id: g.id(),
      name: g.string().resolver('a-field').requires('id')
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User {
        id: ID!
        name: String! @resolver(name: "a-field") @requires(fields: "id")
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

  it('supports federation keys with select', () => {
    g.type('User', {
      id: g.id()
    }).key('id', { select: 'foo(id: $id)' })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @key(fields: "id" resolvable: true select: "foo(id: $id)") {
        id: ID!
      }"
    `)
  })

  it(`supports joins for all the field types`, () => {
    g.type('User', {
      id: g.id().join('foo(id: $id)'),
      str: g.string().join('bar(id: $id)'),
      num: g.boolean().join('baz(id: $id)'),
      list: g.boolean().list().join('bing(id: $id)'),
      generatedType: g.ref('Whatever').join('bazinga(id: $id)')
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User {
        id: ID! @join(select: "foo(id: $id)")
        str: String! @join(select: "bar(id: $id)")
        num: Boolean! @join(select: "baz(id: $id)")
        list: [Boolean!]! @join(select: "bing(id: $id)")
        generatedType: Whatever! @join(select: "bazinga(id: $id)")
      }"
    `)
  })

  it(`supports the deprecated directive`, () => {
    g.type('User', {
      id: g.id().deprecated(),
      num: g.int().deprecated('numbers are for losers')
    })
    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User {
        id: ID! @deprecated
        num: Int! @deprecated(reason: "numbers are for losers")
      }"
    `)
  })

  it(`supports all the federation directives`, () => {
    // There are so many combinations of these - this is not even close to exhaustive
    g.type('User', {
      id: g.id().tag('bloop').tag('bleep').inaccessible(),
      foo: g.int().inaccessible().resolver('a_resolver'),
      bar: g.string().shareable().tag('blah'),
      baz: g.string().override('Products'),
      bez: g.ref('Blah').provides('x y'),
      zip: g.int().optional().inaccessible(),
      zoop: g.int().optional().list().inaccessible(),
      zap: g.int().optional().list().shareable(),
      zoink: g.int().optional().list().tag('foo')
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User {
        id: ID! @tag(name: "bloop") @tag(name: "bleep") @inaccessible
        foo: Int! @inaccessible @resolver(name: "a_resolver")
        bar: String! @shareable @tag(name: "blah")
        baz: String! @override(from: "Products")
        bez: Blah! @provides(fields: "x y")
        zip: Int @inaccessible
        zoop: [Int]! @inaccessible
        zap: [Int]! @shareable
        zoink: [Int]! @tag(name: foo)
      }"
`)
  })
})
