import { config, graph } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Standalone()

describe('Model generator', () => {
  beforeEach(() => g.clear())

  it('generates required String field', () => {
    const model = g.model('User', {
      name: g.string()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: String!
      }"
    `)
  })

  it('generates required ID field', () => {
    const model = g.model('User', {
      identifier: g.id()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        identifier: ID!
      }"
    `)
  })

  it('generates required Int field', () => {
    const model = g.model('User', {
      age: g.int()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        age: Int!
      }"
    `)
  })

  it('generates required Float field', () => {
    const model = g.model('User', {
      weight: g.float()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        weight: Float!
      }"
    `)
  })

  it('generates required Boolean field', () => {
    const model = g.model('User', {
      registered: g.boolean()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        registered: Boolean!
      }"
    `)
  })

  it('generates required Date field', () => {
    const model = g.model('User', {
      birthday: g.date()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        birthday: Date!
      }"
    `)
  })

  it('generates required DateTime field', () => {
    const model = g.model('User', {
      registerationDate: g.datetime()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        registerationDate: DateTime!
      }"
    `)
  })

  it('generates required Enum field', () => {
    const model = g.model('User', {
      email: g.email()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        email: Email!
      }"
    `)
  })

  it('generates required IPAddress field', () => {
    const model = g.model('User', {
      ipAddress: g.ipAddress()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        ipAddress: IPAddress!
      }"
    `)
  })

  it('generates required Timestamp field', () => {
    const model = g.model('User', {
      lastSeen: g.timestamp()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        lastSeen: Timestamp!
      }"
    `)
  })

  it('generates required URL field', () => {
    const model = g.model('User', {
      fediverseInstance: g.url()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        fediverseInstance: URL!
      }"
    `)
  })

  it('generates required JSON field', () => {
    const model = g.model('User', {
      customData: g.json()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        customData: JSON!
      }"
    `)
  })

  it('generates required PhoneNumber field', () => {
    const model = g.model('User', {
      phoneNumber: g.phoneNumber()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        phoneNumber: PhoneNumber!
      }"
    `)
  })

  it('generates more than one field', () => {
    const model = g.model('User', {
      name: g.string(),
      age: g.int()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: String!
        age: Int!
      }"
    `)
  })

  it('generates an optional field', () => {
    const model = g.model('User', {
      name: g.string().optional()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: String
      }"
    `)
  })

  it('generates a required list', () => {
    const model = g.model('User', {
      name: g.string().list()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [String!]!
      }"
    `)
  })

  it('generates a searchable list', () => {
    const model = g.model('User', {
      name: g.string().list().search()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [String!]! @search
      }"
    `)
  })

  it('generates a searchable optional list', () => {
    const model = g.model('User', {
      name: g.string().list().optional().search()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [String!] @search
      }"
    `)
  })

  it('generates a searchable optional list with optional values', () => {
    const model = g.model('User', {
      name: g.string().optional().list().optional().search()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [String] @search
      }"
    `)
  })

  it('generates an optional list with required values', () => {
    const model = g.model('User', {
      name: g.string().list().optional()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [String!]
      }"
    `)
  })

  it('generates an optional list with optional values', () => {
    const model = g.model('User', {
      name: g.string().optional().list().optional()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [String]
      }"
    `)
  })

  it('generates a required list with optional values', () => {
    const model = g.model('User', {
      name: g.string().optional().list()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [String]!
      }"
    `)
  })

  it('generates a searchable model', () => {
    const model = g
      .model('User', {
        name: g.string()
      })
      .search()

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model @search {
        name: String!
      }"
    `)
  })

  it('generates a searchable field', () => {
    const model = g.model('User', {
      name: g.string().search()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @search
      }"
    `)
  })

  it('generates a unique field', () => {
    const model = g.model('User', {
      name: g.string().unique()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @unique
      }"
    `)
  })

  it('generates a unique field with scope', () => {
    const model = g.model('User', {
      name: g.string().unique(['age']),
      age: g.int()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @unique(fields: ["age"])
        age: Int!
      }"
    `)
  })

  it('generates a length with minimum and unique + search', () => {
    const model = g.model('User', {
      name: g.string().length({ min: 2 }).unique().search()
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @length(min: 2) @unique @search
      }"
    `)
  })

  it('generates a length with minimum', () => {
    const model = g.model('User', {
      name: g.string().length({ min: 2 })
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @length(min: 2)
      }"
    `)
  })

  it('generates a length with maximum', () => {
    const model = g.model('User', {
      name: g.string().length({ max: 255 })
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @length(max: 255)
      }"
    `)
  })

  it('generates a length with minimum and maximum', () => {
    const model = g.model('User', {
      name: g.string().length({ min: 2, max: 255 })
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @length(min: 2, max: 255)
      }"
    `)
  })

  it('generates a list length with minimum', () => {
    const model = g.model('User', {
      name: g.string().list().length({ min: 2 })
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [String!]! @length(min: 2)
      }"
    `)
  })

  it('generates a list length with maximum', () => {
    const model = g.model('User', {
      name: g.string().list().length({ max: 255 })
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [String!]! @length(max: 255)
      }"
    `)
  })

  it('generates a list length with minimum and maximum', () => {
    const model = g.model('User', {
      name: g.string().list().length({ min: 2, max: 255 })
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [String!]! @length(min: 2, max: 255)
      }"
    `)
  })

  it('generates a referenced type', () => {
    const address = g.type('Address', {
      street: g.string().optional()
    })

    g.model('User', {
      name: g.string(),
      address: g.ref(address).optional()
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type Address {
        street: String
      }

      type User @model {
        name: String!
        address: Address
      }"
    `)
  })

  it('generates a kitchen sink model', () => {
    const user = g.model('User', {
      name: g
        .string()
        .optional()
        .length({ min: 2 })
        .default('foo')
        .unique()
        .search()
        .auth((rules) => {
          rules.private()
        })
        .cache({ maxAge: 10, staleWhileRevalidate: 5 })
    })

    expect(renderGraphQL(user)).toMatchInlineSnapshot(`
      "type User @model {
        name: String @length(min: 2) @default(value: "foo") @unique @search @auth(rules: [ { allow: private } ]) @cache(maxAge: 10, staleWhileRevalidate: 5)
      }"
    `)
  })

  it('generates an optional referenced type', () => {
    const address = g.type('Address', {
      street: g.string().optional()
    })

    g.model('User', {
      name: g.string(),
      address: g.ref(address).optional()
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type Address {
        street: String
      }

      type User @model {
        name: String!
        address: Address
      }"
    `)
  })

  it('generates a list referenced type', () => {
    const address = g.type('Address', {
      street: g.string().optional()
    })

    g.model('User', {
      name: g.string(),
      addresses: g.ref(address).list()
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type Address {
        street: String
      }

      type User @model {
        name: String!
        addresses: [Address!]!
      }"
    `)
  })

  it('generates a single public auth rule', () => {
    g.model('User', {
      name: g.string()
    }).auth((rules) => {
      rules.public().read()
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model @auth(
          rules: [
            { allow: public, operations: [read] }
          ]) {
        name: String!
      }"
    `)
  })

  it('generates a single auth rule', () => {
    g.model('User', {
      name: g.string()
    }).auth((rules) => {
      rules.private().read()
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model @auth(
          rules: [
            { allow: private, operations: [read] }
          ]) {
        name: String!
      }"
    `)
  })

  it('generates a multiple auth rules', () => {
    g.model('User', {
      name: g.string()
    }).auth((rules) => {
      rules.private().read()
      rules.groups(['admin'])
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model @auth(
          rules: [
            { allow: private, operations: [read] }
            { allow: groups, groups: ["admin"] }
          ]) {
        name: String!
      }"
    `)
  })

  it('generates a field with a public auth rule', () => {
    g.model('User', {
      name: g.string().auth((rules) => {
        rules.public()
      })
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @auth(rules: [ { allow: public } ])
      }"
    `)
  })

  it('generates a field with a single auth rule', () => {
    g.model('User', {
      name: g.string().auth((rules) => {
        rules.groups(['admin'])
      })
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @auth(rules: [ { allow: groups, groups: ["admin"] } ])
      }"
    `)
  })

  it('generates a unique field with a single auth rule', () => {
    g.model('User', {
      name: g
        .string()
        .unique()
        .auth((rules) => {
          rules.groups(['admin'])
        })
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @unique @auth(rules: [ { allow: groups, groups: ["admin"] } ])
      }"
    `)
  })

  it('generates a field with a default value and a single auth rule', () => {
    g.model('User', {
      age: g
        .int()
        .default(1)
        .auth((rules) => {
          rules.private()
        })
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        age: Int! @default(value: 1) @auth(rules: [ { allow: private } ])
      }"
    `)
  })

  it('generates a searchable field with a single auth rule', () => {
    g.model('User', {
      name: g
        .string()
        .search()
        .auth((rules) => {
          rules.private()
        })
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @search @auth(rules: [ { allow: private } ])
      }"
    `)
  })

  it('generates a composite type field with a single auth rule', () => {
    const address = g.type('Address', {
      street: g.string().optional()
    })

    g.model('User', {
      address: g.ref(address).auth((rules) => {
        rules.private()
      })
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type Address {
        street: String
      }

      type User @model {
        address: Address! @auth(rules: [ { allow: private } ])
      }"
    `)
  })

  it('generates a list field with a single auth rule', () => {
    g.model('User', {
      cats: g
        .string()
        .list()
        .optional()
        .auth((rules) => rules.private())
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        cats: [String!] @auth(rules: [ { allow: private } ])
      }"
    `)
  })

  it('generates a length-limited field with a single auth rule', () => {
    g.model('User', {
      cats: g
        .string()
        .length({ min: 2 })
        .auth((rules) => rules.private())
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        cats: String! @length(min: 2) @auth(rules: [ { allow: private } ])
      }"
    `)
  })

  it('generates a relation field with a single auth rule', () => {
    const model = g.model('User', {
      self: g.relation(() => model).auth((rules) => rules.private())
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        self: User! @auth(rules: [ { allow: private } ])
      }"
    `)
  })

  it('generates a relation list field with a single auth rule', () => {
    const model = g.model('User', {
      self: g
        .relation(() => model)
        .list()
        .auth((rules) => rules.private())
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        self: [User!]! @auth(rules: [ { allow: private } ])
      }"
    `)
  })

  it('generates a resolver to a field', () => {
    g.model('User', {
      name: g.string().resolver('a-field')
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @resolver(name: "a-field")
      }"
    `)
  })

  it('generates a composite type field with a resolver', () => {
    const address = g.type('Address', {
      street: g.string().optional()
    })

    g.model('User', {
      name: g.ref(address).resolver('a-field')
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type Address {
        street: String
      }

      type User @model {
        name: Address! @resolver(name: "a-field")
      }"
    `)
  })

  it('generates a list field with a resolver', () => {
    g.model('User', {
      name: g.string().list().resolver('a-field')
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        name: [String!]! @resolver(name: "a-field")
      }"
    `)
  })

  it('generates a model level cache', () => {
    g.model('User', {
      name: g.string().optional()
    }).cache({ maxAge: 60 })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model @cache(maxAge: 60) {
        name: String
      }"
    `)
  })

  it('generates a model level cache with staleWhileRevalidate', () => {
    g.model('User', {
      name: g.string().optional()
    }).cache({ maxAge: 60, staleWhileRevalidate: 50 })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model @cache(maxAge: 60, staleWhileRevalidate: 50) {
        name: String
      }"
    `)
  })

  it('generates a model level cache with type mutation invalidation', () => {
    g.model('User', {
      name: g.string().optional()
    }).cache({ maxAge: 60, mutationInvalidation: 'type' })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model @cache(maxAge: 60, mutationInvalidation: type) {
        name: String
      }"
    `)
  })

  it('generates a model level cache with entity mutation invalidation', () => {
    g.model('User', {
      name: g.string().optional()
    }).cache({ maxAge: 60, mutationInvalidation: 'entity' })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model @cache(maxAge: 60, mutationInvalidation: entity) {
        name: String
      }"
    `)
  })

  it('generates a model level cache with list mutation invalidation', () => {
    g.model('User', {
      name: g.string().optional()
    }).cache({ maxAge: 60, mutationInvalidation: 'list' })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model @cache(maxAge: 60, mutationInvalidation: list) {
        name: String
      }"
    `)
  })

  it('generates a model level cache with custom field mutation invalidation', () => {
    g.model('User', {
      name: g.string().optional()
    }).cache({ maxAge: 60, mutationInvalidation: { field: 'name' } })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model @cache(maxAge: 60, mutationInvalidation: { field: "name" }) {
        name: String
      }"
    `)
  })

  it('generates a field level cache', () => {
    g.model('User', {
      name: g
        .string()
        .optional()
        .cache({ maxAge: 60, staleWhileRevalidate: 50 })
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        name: String @cache(maxAge: 60, staleWhileRevalidate: 50)
      }"
    `)
  })

  it('generates a cache with unique', () => {
    g.model('User', {
      name: g.string().optional().unique().cache({ maxAge: 60 })
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        name: String @unique @cache(maxAge: 60)
      }"
    `)
  })

  it('generates a cache with default', () => {
    g.model('User', {
      name: g
        .string()
        .optional()
        .default('Bob')
        .cache({ maxAge: 60, staleWhileRevalidate: 50 })
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        name: String @default(value: "Bob") @cache(maxAge: 60, staleWhileRevalidate: 50)
      }"
    `)
  })

  it('generates a cache with length-limited string', () => {
    g.model('User', {
      name: g
        .string()
        .optional()
        .length({ min: 1 })
        .cache({ maxAge: 60, staleWhileRevalidate: 50 })
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        name: String @length(min: 1) @cache(maxAge: 60, staleWhileRevalidate: 50)
      }"
    `)
  })

  it('generates a cache with resolver', () => {
    g.model('User', {
      name: g
        .string()
        .optional()
        .resolver('a-field')
        .cache({ maxAge: 60, staleWhileRevalidate: 50 })
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        name: String @resolver(name: "a-field") @cache(maxAge: 60, staleWhileRevalidate: 50)
      }"
    `)
  })

  it('generates a cache with search', () => {
    g.model('User', {
      name: g
        .string()
        .optional()
        .search()
        .cache({ maxAge: 60, staleWhileRevalidate: 50 })
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type User @model {
        name: String @search @cache(maxAge: 60, staleWhileRevalidate: 50)
      }"
    `)
  })

  it('prevents using of whitespaced identifier as the name', () => {
    expect(() => g.model('white space', { name: g.string() })).toThrow(
      'Given name "white space" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of number-prefixed identifier as the name', () => {
    expect(() => g.model('0User', { name: g.string() })).toThrow(
      'Given name "0User" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of weird characters identifier as the name', () => {
    expect(() => g.model('!@#$%^&*()+|~`=-', { name: g.string() })).toThrow(
      'Given name "!@#$%^&*()+|~`=-" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of whitespaced identifier as a field name', () => {
    expect(() => g.model('A', { 'white space': g.string() })).toThrow(
      'Given name "white space" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of number-prefixed identifier as a field name', () => {
    expect(() => g.model('A', { '0name': g.string() })).toThrow(
      'Given name "0name" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of weird characters identifier as a field name', () => {
    expect(() => g.model('A', { '!@#$%^&*()+|~`=-': g.string() })).toThrow(
      'Given name "!@#$%^&*()+|~`=-" is not a valid TypeScript identifier.'
    )
  })
})
