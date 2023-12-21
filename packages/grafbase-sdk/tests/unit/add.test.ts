import { config, graph, scalar, define } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Standalone()

describe('Add, define, scalar', () => {
  beforeEach(() => g.clear())

  it('correctly produces a schema built using scalar, define and add', () => {
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
