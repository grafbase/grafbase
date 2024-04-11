import { SubscriptionTransport, config, graph } from '../../src/index'
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

  it('supports subscription settings', () => {
    const cfg = config({
      graph: graph.Federated({
        subscriptions: (subscriptions) => {
          subscriptions
            .subgraph('Product')
            .transport(SubscriptionTransport.GraphQlOverWebsockets, {
              url: 'http://example.com'
            })
        }
      })
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "
      extend schema
        @graph(type: federated)
        @subgraph(name: "Product", websocketUrl: "http://example.com")
      "
    `)
  })

  it('supports cache configuration', () => {
    const cfg = config({
      graph: graph.Federated({
        cache: {
          rules: [
            {
              types: 'Query',
              maxAge: 60
            }
          ]
        }
      })
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "
      extend schema
        @graph(type: federated)
      extend schema
        @cache(rules: [
          {
            types: "Query",
            maxAge: 60
          }
        ])

      "
    `)
  })

  it('supports subgraph development URLs', () => {
    const cfg = config({
      graph: graph.Federated({
        subgraphs: [
          { name: 'Product', url: 'http://example.com/product' },
          { name: 'Review', url: 'http://example.com/review' }
        ],
        headers: (headers) => {
          headers.subgraph('Product').set('Bloop', 'Bleep')

          headers.subgraph('Review').set('Bloop', 'Bleep')
        }
      })
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "
      extend schema
        @graph(type: federated)
        @subgraph(name: "Product", developmentUrl: "http://example.com/product")
        @subgraph(name: "Review", developmentUrl: "http://example.com/review")
        @subgraph(name: "Product", headers: [{ name: "Bloop", value: "Bleep" }]),
        @subgraph(name: "Review", headers: [{ name: "Bloop", value: "Bleep" }])
      "
    `)
  })
})
