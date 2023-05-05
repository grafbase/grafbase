import { config, g } from '../../src/index'
import { describe, expect, it } from '@jest/globals'

describe('Model generator', () => {
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

    const user = g.model('User', {
      name: g.string(),
      address: g.ref(address)
    })

    const cfg = config().schema({
      types: [address],
      models: [user]
    })

    expect(cfg.toString()).toMatchInlineSnapshot(`
      "type Address {
        street: String
      }

      type User @model {
        name: String!
        address: Address!
      }"
    `)
  })

  it('generates a kitchen sink model', () => {
    const user = g.model('User', {
      name: g.string().length({ min: 2 }).default('foo').unique().search()
    })

    expect(user.toString()).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @length(min: 2) @default(value: "foo") @unique @search
      }"
    `)
  })

  it('generates an optional referenced type', () => {
    const address = g.type('Address', {
      street: g.string().optional()
    })

    const user = g.model('User', {
      name: g.string(),
      address: g.ref(address).optional()
    })

    const cfg = config().schema({
      types: [address],
      models: [user]
    })

    expect(cfg.toString()).toMatchInlineSnapshot(`
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

    const user = g.model('User', {
      name: g.string(),
      addresses: g.ref(address).list()
    })

    const cfg = config().schema({
      types: [address],
      models: [user]
    })

    expect(cfg.toString()).toMatchInlineSnapshot(`
      "type Address {
        street: String
      }

      type User @model {
        name: String!
        addresses: [Address!]!
      }"
    `)
  })
})
