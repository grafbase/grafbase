import { config, g } from '../../src/index'
import { describe, expect, it } from '@jest/globals'

describe('Query generator', () => {
  it('generates a resolver with required input and output', () => {
    const greetQuery = g
      .query('greet', {
        args: { name: g.string() },
        returns: g.string()
      })
      .resolver('hello')

    expect(greetQuery.toString()).toMatchInlineSnapshot(`
      "extend type Query {
        greet(name: String!): String! @resolver(name: "hello")
      }"
    `)
  })

  it('generates a resolver with optional input', () => {
    const greetQuery = g
      .query('greet', {
        args: { name: g.string().optional() },
        returns: g.string()
      })
      .resolver('hello')

    expect(greetQuery.toString()).toMatchInlineSnapshot(`
      "extend type Query {
        greet(name: String): String! @resolver(name: "hello")
      }"
    `)
  })

  it('generates a resolver with optional input and output', () => {
    const greetQuery = g
      .query('greet', {
        args: { name: g.string().optional() },
        returns: g.string().optional()
      })
      .resolver('hello')

    expect(greetQuery.toString()).toMatchInlineSnapshot(`
      "extend type Query {
        greet(name: String): String @resolver(name: "hello")
      }"
    `)
  })

  it('generates a resolver with list input', () => {
    const greetQuery = g
      .query('greet', {
        args: { name: g.string().list() },
        returns: g.string()
      })
      .resolver('hello')

    expect(greetQuery.toString()).toMatchInlineSnapshot(`
      "extend type Query {
        greet(name: [String!]!): String! @resolver(name: "hello")
      }"
    `)
  })

  it('generates a resolver with list output', () => {
    const greetQuery = g
      .query('greet', {
        args: { name: g.string() },
        returns: g.string().list()
      })
      .resolver('hello')

    expect(greetQuery.toString()).toMatchInlineSnapshot(`
      "extend type Query {
        greet(name: String!): [String!]! @resolver(name: "hello")
      }"
    `)
  })

  it('generates a resolver with list output', () => {
    const greetQuery = g
      .query('greet', {
        args: { name: g.string() },
        returns: g.string().list()
      })
      .resolver('hello')

    expect(greetQuery.toString()).toMatchInlineSnapshot(`
      "extend type Query {
        greet(name: String!): [String!]! @resolver(name: "hello")
      }"
    `)
  })

  it('generates a mutation resolver with required input and output', () => {
    const input = g.type('CheckoutSessionInput', { name: g.string() })
    const output = g.type('CheckoutSessionOutput', { successful: g.boolean() })

    const checkout = g
      .mutation('checkout', {
        args: { input: g.ref(input) },
        returns: g.ref(output)
      })
      .resolver('checkout')

    const cfg = config().schema({
      types: [input, output],
      queries: [checkout]
    })

    expect(cfg.toString()).toMatchInlineSnapshot(`
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
    const enm = g.enumType('Foo', ['Bar', 'Baz'])

    const greetQuery = g
      .query('greet', {
        args: { name: g.string() },
        returns: g.string().list()
      })
      .resolver('hello')

    const sweepQuery = g
      .query('greet', {
        args: { game: g.int().optional() },
        returns: g.enum(enm).list()
      })
      .resolver('hello')

    const cfg = config().schema({
      queries: [greetQuery, sweepQuery],
      enums: [enm]
    })

    expect(cfg.toString()).toMatchInlineSnapshot(`
      "enum Foo {
        Bar,
        Baz
      }

      extend type Query {
        greet(name: String!): [String!]! @resolver(name: "hello")
      }

      extend type Query {
        greet(game: Int): [Foo!]! @resolver(name: "hello")
      }"
    `)
  })
})
