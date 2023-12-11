import { config, graph } from '../../src/index'
import { describe, expect, it } from '@jest/globals'
import { renderGraphQL } from '../utils'

describe('Federation config', () => {
  it('renders a graph directive that extends the schema', () => {
    const cfg = config({
      graph: graph.Federated()
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "
      extend schema
        @graph(type: federated)
      "
    `)
  })

  it('supports subgraph and default headers', () => {
    const cfg = config({
      graph: graph.Federated({
        headers: (headers) => {
          headers.set('Foo', 'Bar')
          headers.set('Forward', { forward: 'Source' })

          headers
            .subgraph('Product')
            .set('Authorization', { forward: 'Authorization' })
            .set('Bloop', 'Bleep')

          headers.subgraph('Review').set('Bloop', 'Bleep')

          headers.subgraph('Product').set('AnotherOne', 'Meep')
        }
      })
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "
      extend schema
        @graph(type: federated)
        @allSubgraphs(headers: [{ name: "Foo", value: "Bar" }, { name: "Forward", forward: "Source" }])
        @subgraph(name: "Product", headers: [{ name: "Authorization", forward: "Authorization" }, { name: "Bloop", value: "Bleep" }, { name: "AnotherOne", value: "Meep" }]),
        @subgraph(name: "Review", headers: [{ name: "Bloop", value: "Bleep" }])
      "
    `)
  })
})
