import { config, g } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'

describe('Model generator', () => {
  beforeEach(() => g.clear())

  it('generates required String field', () => {
    const model = g.model('User', {
      name: g.string()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: String!
      }"
    `)
  })

  it('generates required ID field', () => {
    const model = g.model('User', {
      identifier: g.id()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        identifier: ID!
      }"
    `)
  })

  it('generates required Int field', () => {
    const model = g.model('User', {
      age: g.int()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        age: Int!
      }"
    `)
  })

  it('generates required Float field', () => {
    const model = g.model('User', {
      weight: g.float()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        weight: Float!
      }"
    `)
  })

  it('generates required Boolean field', () => {
    const model = g.model('User', {
      registered: g.boolean()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        registered: Boolean!
      }"
    `)
  })

  it('generates required Date field', () => {
    const model = g.model('User', {
      birthday: g.date()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        birthday: Date!
      }"
    `)
  })

  it('generates required DateTime field', () => {
    const model = g.model('User', {
      registerationDate: g.datetime()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        registerationDate: DateTime!
      }"
    `)
  })

  it('generates required Enum field', () => {
    const model = g.model('User', {
      email: g.email()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        email: Email!
      }"
    `)
  })

  it('generates required IPAddress field', () => {
    const model = g.model('User', {
      ipAddress: g.ipAddress()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        ipAddress: IPAddress!
      }"
    `)
  })

  it('generates required Timestamp field', () => {
    const model = g.model('User', {
      lastSeen: g.timestamp()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        lastSeen: Timestamp!
      }"
    `)
  })

  it('generates required URL field', () => {
    const model = g.model('User', {
      fediverseInstance: g.url()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        fediverseInstance: URL!
      }"
    `)
  })

  it('generates required JSON field', () => {
    const model = g.model('User', {
      customData: g.json()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        customData: JSON!
      }"
    `)
  })

  it('generates required PhoneNumber field', () => {
    const model = g.model('User', {
      phoneNumber: g.phoneNumber()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
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

    expect(model.toString()).toMatchInlineSnapshot(`
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

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: String
      }"
    `)
  })

  it('generates a required list', () => {
    const model = g.model('User', {
      name: g.string().list()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: [String!]!
      }"
    `)
  })

  it('generates a searchable list', () => {
    const model = g.model('User', {
      name: g.string().list().search()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: [String!]! @search
      }"
    `)
  })

  it('generates a searchable optional list', () => {
    const model = g.model('User', {
      name: g.string().list().optional().search()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: [String!] @search
      }"
    `)
  })

  it('generates a searchable optional list with optional values', () => {
    const model = g.model('User', {
      name: g.string().optional().list().optional().search()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: [String] @search
      }"
    `)
  })

  it('generates an optional list with required values', () => {
    const model = g.model('User', {
      name: g.string().list().optional()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: [String!]
      }"
    `)
  })

  it('generates an optional list with optional values', () => {
    const model = g.model('User', {
      name: g.string().optional().list().optional()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: [String]
      }"
    `)
  })

  it('generates a required list with optional values', () => {
    const model = g.model('User', {
      name: g.string().optional().list()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
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

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model @search {
        name: String!
      }"
    `)
  })

  it('generates a searchable field', () => {
    const model = g.model('User', {
      name: g.string().search()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @search
      }"
    `)
  })

  it('generates a unique field', () => {
    const model = g.model('User', {
      name: g.string().unique()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
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

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @unique(fields: ["age"])
        age: Int!
      }"
    `)
  })

  it('generates a live model', () => {
    const model = g
      .model('User', {
        name: g.string()
      })
      .live()

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model @live {
        name: String!
      }"
    `)
  })

  it('generates a length with minimum', () => {
    const model = g.model('User', {
      name: g.string().length({ min: 2 })
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @length(min: 2)
      }"
    `)
  })

  it('generates a length with minimum and unique + search', () => {
    const model = g.model('User', {
      name: g.string().length({ min: 2 }).unique().search()
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @length(min: 2) @unique @search
      }"
    `)
  })

  it('generates a length with maximum', () => {
    const model = g.model('User', {
      name: g.string().length({ max: 255 })
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @length(max: 255)
      }"
    `)
  })

  it('generates a length with minimum and maximum', () => {
    const model = g.model('User', {
      name: g.string().length({ min: 2, max: 255 })
    })

    expect(model.toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @length(min: 2, max: 255)
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

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
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
        .length({ min: 2 })
        .default('foo')
        .unique()
        .search()
        .auth((rules) => {
          rules.private()
        })
    })

    expect(user.toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @length(min: 2) @default(value: "foo") @unique @search @auth(rules: [ { allow: private } ])
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

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
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

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type Address {
        street: String
      }

      type User @model {
        name: String!
        addresses: [Address!]!
      }"
    `)
  })

  it('generates a single auth rule', () => {
    g.model('User', {
      name: g.string()
    }).auth((rules) => {
      rules.private().read()
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
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

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type User @model @auth(
          rules: [
            { allow: private, operations: [read] }
            { allow: groups, groups: ["admin"] }
          ]) {
        name: String!
      }"
    `)
  })

  it('generates a field with a single auth rule', () => {
    g.model('User', {
      name: g.string().auth((rules) => {
        rules.groups(['admin'])
      })
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
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

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
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

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
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

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
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

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
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

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
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

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type User @model {
        cats: String! @length(min: 2) @auth(rules: [ { allow: private } ])
      }"
    `)
  })

  it('generates a relation field with a single auth rule', () => {
    const model = g.model('User', {
      self: g.relation(() => model).auth((rules) => rules.private())
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
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

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type User @model {
        self: [User!]! @auth(rules: [ { allow: private } ])
      }"
    `)
  })

  it('generates a resolver to a field', () => {
    g.model('User', {
      name: g.string().resolver('a-field')
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @resolver(name: "a-field")
      }"
    `)
  })

  it('generates a unique field with a resolver', () => {
    g.model('User', {
      name: g.string().unique().resolver('a-field')
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @unique @resolver(name: "a-field")
      }"
    `)
  })

  it('generates a field with a default value and a resolver', () => {
    g.model('User', {
      name: g.string().default('Bob').resolver('a-field')
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @default(value: "Bob") @resolver(name: "a-field")
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

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
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

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: [String!]! @resolver(name: "a-field")
      }"
    `)
  })
})
