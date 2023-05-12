import { config, g } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'

describe('Interface generator', () => {
  beforeEach(() => g.clear())

  it('generates a simple interface', () => {
    const i = g.interface('Produce', {
      name: g.string(),
      quantity: g.int(),
      price: g.float(),
      nutrients: g.string().optional().list().optional()
    })

    expect(i.toString()).toMatchInlineSnapshot(`
      "interface Produce {
        name: String!
        quantity: Int!
        price: Float!
        nutrients: [String]
      }"
    `)
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

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
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
})
