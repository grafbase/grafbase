import { config, graph, g } from '../../src/index'
import { describe, expect, it } from '@jest/globals'
import { renderGraphQL } from '../utils'

describe('Federation config', () => {
  it('renders a graph directive that extends the schema', () => {
    const cfg = config({
      graph: graph.Federation()
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "
      extend schema @graph(type: federated)
      "
  `)
  })
})
