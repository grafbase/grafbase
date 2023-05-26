import { config, g } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

describe('Query generator', () => {
  beforeEach(() => g.clear())

  it('generates a resolver with empty args', () => {
    g.query('greet', {
      returns: g.string(),
      resolver: 'hello'
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend type Query {
        greet: String! @resolver(name: "hello")
      }"
    `)
  })

  it('generates a resolver with required input and output', () => {
    g.query('greet', {
      args: { name: g.string() },
      returns: g.string(),
      resolver: 'hello'
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend type Query {
        greet(name: String!): String! @resolver(name: "hello")
      }"
    `)
  })

  it('generates a resolver with optional input', () => {
    g.query('greet', {
      args: { name: g.string().optional() },
      returns: g.string(),
      resolver: 'hello'
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend type Query {
        greet(name: String): String! @resolver(name: "hello")
      }"
    `)
  })

  it('generates a resolver with optional input and output', () => {
    g.query('greet', {
      args: { name: g.string().optional() },
      returns: g.string().optional(),
      resolver: 'hello'
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend type Query {
        greet(name: String): String @resolver(name: "hello")
      }"
    `)
  })

  it('generates a resolver with list input', () => {
    g.query('greet', {
      args: { name: g.string().list() },
      returns: g.string(),
      resolver: 'hello'
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend type Query {
        greet(name: [String!]!): String! @resolver(name: "hello")
      }"
    `)
  })

  it('generates a resolver with list output', () => {
    g.query('greet', {
      args: { name: g.string() },
      returns: g.string().list(),
      resolver: 'hello'
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend type Query {
        greet(name: String!): [String!]! @resolver(name: "hello")
      }"
    `)
  })

  it('generates a resolver with list output', () => {
    g.query('greet', {
      args: { name: g.string() },
      returns: g.string().list(),
      resolver: 'hello'
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend type Query {
        greet(name: String!): [String!]! @resolver(name: "hello")
      }"
    `)
  })

  it('generates a mutation resolver with required input and output', () => {
    const input = g.type('CheckoutSessionInput', { name: g.string() })
    const output = g.type('CheckoutSessionOutput', { successful: g.boolean() })

    g.mutation('checkout', {
      args: { input: g.ref(input) },
      returns: g.ref(output),
      resolver: 'checkout'
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "type CheckoutSessionInput {
        name: String!
      }

      type CheckoutSessionOutput {
        successful: Boolean!
      }

      extend type Mutation {
        checkout(input: CheckoutSessionInput!): CheckoutSessionOutput! @resolver(name: "checkout")
      }"
    `)
  })

  it('generates a query as part of the full SDL', () => {
    const enm = g.enum('Foo', ['Bar', 'Baz'])

    g.query('greet', {
      args: { name: g.string() },
      returns: g.string().list(),
      resolver: 'hello'
    })

    g.query('sweet', {
      args: { game: g.int().optional() },
      returns: g.enumRef(enm).list(),
      resolver: 'jello'
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "enum Foo {
        Bar,
        Baz
      }

      extend type Query {
        greet(name: String!): [String!]! @resolver(name: "hello")
        sweet(game: Int): [Foo!]! @resolver(name: "jello")
      }"
    `)
  })

  it('prevents using of whitespaced identifier as the name', () => {
    expect(() =>
      g.query('white space', {
        args: { name: g.string() },
        returns: g.string(),
        resolver: 'hello'
      })
    ).toThrow('Given name "white space" is not a valid TypeScript identifier.')
  })

  it('prevents using of number-prefixed identifier as the name', () => {
    expect(() =>
      g.query('0User', {
        args: { name: g.string() },
        returns: g.string(),
        resolver: 'hello'
      })
    ).toThrow('Given name "0User" is not a valid TypeScript identifier.')
  })

  it('prevents using of weird characters identifier as the name', () => {
    expect(() =>
      g.query('!@#$%^&*()+|~`=-', {
        args: { name: g.string() },
        returns: g.string(),
        resolver: 'hello'
      })
    ).toThrow(
      'Given name "!@#$%^&*()+|~`=-" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of whitespaced identifier an argument name', () => {
    expect(() =>
      g.query('Test', {
        args: { 'white space': g.string() },
        returns: g.string(),
        resolver: 'hello'
      })
    ).toThrow('Given name "white space" is not a valid TypeScript identifier.')
  })

  it('prevents using of number-prefixed identifier as an argument name', () => {
    expect(() =>
      g.query('Test', {
        args: { '0name': g.string() },
        returns: g.string(),
        resolver: 'hello'
      })
    ).toThrow('Given name "0name" is not a valid TypeScript identifier.')
  })

  it('prevents using of weird characters identifier as an argument name', () => {
    expect(() =>
      g.query('Test', {
        args: { '!@#$%^&*()+|~`=-': g.string() },
        returns: g.string(),
        resolver: 'hello'
      })
    ).toThrow(
      'Given name "!@#$%^&*()+|~`=-" is not a valid TypeScript identifier.'
    )
  })
})
