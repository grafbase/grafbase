import { config, graph } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Single()

describe('Interface generator', () => {
  beforeEach(() => g.clear())

  it('generates a simple interface', () => {
    const i = g.interface('Produce', {
      name: g.string(),
      quantity: g.int(),
      price: g.float(),
      nutrients: g.string().optional().list().optional()
    })

    expect(renderGraphQL(i)).toMatchInlineSnapshot(`
      "interface Produce {
        name: String!
        quantity: Int!
        price: Float!
        nutrients: [String]
      }"
    `)
  })

  it('prevents using of whitespaced identifier as the name', () => {
    expect(() => g.interface('white space', { name: g.string() })).toThrow(
      'Given name "white space" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of number-prefixed identifier as the name', () => {
    expect(() => g.interface('0User', { name: g.string() })).toThrow(
      'Given name "0User" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of weird characters identifier as the name', () => {
    expect(() => g.interface('!@#$%^&*()+|~`=-', { name: g.string() })).toThrow(
      'Given name "!@#$%^&*()+|~`=-" is not a valid TypeScript identifier.'
    )
  })

  it('generates a type implementing an interface', () => {
    const produce = g.interface('Produce', {
      name: g.string(),
      quantity: g.int(),
      price: g.float(),
      nutrients: g.string().optional().list().optional()
    })

    g.type('Fruit', {
      isSeedless: g.boolean().optional(),
      ripenessIndicators: g.string().optional().list().optional()
    }).implements(produce)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "interface Produce {
        name: String!
        quantity: Int!
        price: Float!
        nutrients: [String]
      }

      type Fruit implements Produce {
        name: String!
        quantity: Int!
        price: Float!
        nutrients: [String]
        isSeedless: Boolean
        ripenessIndicators: [String]
      }"
    `)
  })

  it('generates a type implementing multiple interfaces', () => {
    const produce = g.interface('Produce', {
      name: g.string()
    })

    const sweets = g.interface('Sweets', {
      name: g.string(),
      sweetness: g.int()
    })

    g.type('Fruit', {
      isSeedless: g.boolean().optional(),
      ripenessIndicators: g.string().optional().list().optional()
    })
      .implements(produce)
      .implements(sweets)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "interface Produce {
        name: String!
      }

      interface Sweets {
        name: String!
        sweetness: Int!
      }

      type Fruit implements Produce & Sweets {
        name: String!
        sweetness: Int!
        isSeedless: Boolean
        ripenessIndicators: [String]
      }"
    `)
  })
})
