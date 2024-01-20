import { config, graph } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Standalone()

describe('Operation limits', () => {
  beforeEach(() => g.clear())

  it('renders the defined operation limit values', async () => {
    const cfg = config({
      graph: g,
      operationLimits: {
        complexity: 100
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @operationLimits(complexity: 100)

      "
    `)
  })


  it('renders the defined multiple operation limit values', async () => {
    const cfg = config({
      graph: g,
      operationLimits: {
        complexity: 100,
        depth: 5
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @operationLimits(complexity: 100, depth: 5)

      "
    `)
  })

  it('does not render anything if no operation limits provided', async () => {
    const _ = g.type('User', {
      name: g.string()
    })
    const cfg = config({
      graph: g,
      operationLimits: {}
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "type User {
        name: String!
      }"
    `)
  })
})
