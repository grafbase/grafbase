import { graph } from '../../src/index'
import { describe, expect, it } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Single()

describe('Default value generation', () => {
  it('generates String default', () => {
    const model = g.model('User', {
      name: g.string().default('Bob')
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: String! @default(value: "Bob")
      }"
    `)
  })

  it('generates String default as optional', () => {
    const model = g.model('User', {
      name: g.string().optional().default('Bob')
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: String @default(value: "Bob")
      }"
    `)
  })

  it('generates String list default', () => {
    const model = g.model('User', {
      name: g.string().list().default(['Bob'])
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [String!]! @default(value: ["Bob"])
      }"
    `)
  })

  it('generates ID default', () => {
    const model = g.model('User', {
      name: g.id().default('asdf123')
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: ID! @default(value: "asdf123")
      }"
    `)
  })

  it('generates ID list default', () => {
    const model = g.model('User', {
      name: g.id().list().default(['asdf123', 'omg123'])
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [ID!]! @default(value: ["asdf123", "omg123"])
      }"
    `)
  })

  it('generates PhoneNumber default', () => {
    const model = g.model('User', {
      name: g.phoneNumber().default('555 123 123')
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: PhoneNumber! @default(value: "555 123 123")
      }"
    `)
  })

  it('generates PhoneNumber list default', () => {
    const model = g.model('User', {
      name: g.phoneNumber().list().default(['555 123 123', '555 432 432'])
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [PhoneNumber!]! @default(value: ["555 123 123", "555 432 432"])
      }"
    `)
  })

  it('generates IPAddress default', () => {
    const model = g.model('User', {
      name: g.ipAddress().default('0.0.0.0')
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: IPAddress! @default(value: "0.0.0.0")
      }"
    `)
  })

  it('generates IPAddress list default', () => {
    const model = g.model('User', {
      name: g.ipAddress().list().default(['0.0.0.0', '1.1.1.1'])
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [IPAddress!]! @default(value: ["0.0.0.0", "1.1.1.1"])
      }"
    `)
  })

  it('generates Email default', () => {
    const model = g.model('User', {
      name: g.email().default('foo@bar.lol')
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: Email! @default(value: "foo@bar.lol")
      }"
    `)
  })

  it('generates Email list default', () => {
    const model = g.model('User', {
      name: g.email().list().default(['foo@bar.lol'])
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [Email!]! @default(value: ["foo@bar.lol"])
      }"
    `)
  })

  it('generates URL default', () => {
    const model = g.model('User', {
      name: g.url().default('https://github.com')
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: URL! @default(value: "https://github.com")
      }"
    `)
  })

  it('generates URL list default', () => {
    const model = g.model('User', {
      name: g
        .url()
        .list()
        .default(['https://github.com', 'https://codeberg.org'])
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [URL!]! @default(value: ["https://github.com", "https://codeberg.org"])
      }"
    `)
  })

  it('generates Int default', () => {
    const model = g.model('User', {
      name: g.int().default(2)
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: Int! @default(value: 2)
      }"
    `)
  })

  it('generates Int list default', () => {
    const model = g.model('User', {
      name: g.int().list().default([2])
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [Int!]! @default(value: [2])
      }"
    `)
  })

  it('generates Float default', () => {
    const model = g.model('User', {
      name: g.float().default(1.337)
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: Float! @default(value: 1.337)
      }"
    `)
  })

  it('generates Float list default', () => {
    const model = g.model('User', {
      name: g.float().list().default([1.337, 2.32])
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [Float!]! @default(value: [1.337, 2.32])
      }"
    `)
  })

  it('generates Boolean default', () => {
    const model = g.model('User', {
      name: g.boolean().default(false)
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: Boolean! @default(value: false)
      }"
    `)
  })

  it('generates Boolean list default', () => {
    const model = g.model('User', {
      name: g.boolean().list().default([true, false])
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        name: [Boolean!]! @default(value: [true, false])
      }"
    `)
  })

  it('generates Date default', () => {
    const model = g.model('User', {
      date: g.date().default(new Date('1995-12-17'))
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        date: Date! @default(value: "1995-12-17")
      }"
    `)
  })

  it('generates Date list default', () => {
    const model = g.model('User', {
      date: g
        .date()
        .list()
        .default([new Date('1995-12-17'), new Date('2002-01-01')])
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        date: [Date!]! @default(value: ["1995-12-17", "2002-01-01"])
      }"
    `)
  })

  it('generates DateTime default', () => {
    const model = g.model('User', {
      date: g.datetime().default(new Date('1995-12-17T04:20:00Z'))
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        date: DateTime! @default(value: "1995-12-17T04:20:00Z")
      }"
    `)
  })

  it('generates DateTime list default', () => {
    const model = g.model('User', {
      date: g
        .datetime()
        .list()
        .default([
          new Date('1995-12-17T04:20:00Z'),
          new Date('2002-12-17T04:20:00Z')
        ])
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        date: [DateTime!]! @default(value: ["1995-12-17T04:20:00Z", "2002-12-17T04:20:00Z"])
      }"
    `)
  })

  it('generates Timestamp default', () => {
    const model = g.model('User', {
      date: g.timestamp().default(1683644443566)
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        date: Timestamp! @default(value: 1683644443566)
      }"
    `)
  })

  it('generates Timestamp list default', () => {
    const model = g.model('User', {
      date: g.timestamp().list().default([1683644443566, 1683644443569])
    })

    expect(renderGraphQL(model)).toMatchInlineSnapshot(`
      "type User @model {
        date: [Timestamp!]! @default(value: [1683644443566, 1683644443569])
      }"
    `)
  })
})
