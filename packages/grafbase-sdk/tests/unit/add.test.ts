import { config, graph, scalar, define } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Standalone()

describe('Interface generator', () => {
  beforeEach(() => g.clear())

  it('generates a type implementing multiple interfaces', () => {
    const produce = define.interface('Produce', {
      name: scalar.string()
    })

    const sweets = define.interface('Sweets', {
      name: scalar.string(),
      sweetness: scalar.int()
    })

    const fruit = define
      .type('Fruit', {
        isSeedless: scalar.boolean().optional(),
        ripenessIndicators: scalar.string().optional().list().optional()
      })
      .implements(produce)
      .implements(sweets)

    g.add(produce, sweets, fruit)

    expect(renderGraphQL(config({ graph: g }))).toMatchInlineSnapshot(`
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
