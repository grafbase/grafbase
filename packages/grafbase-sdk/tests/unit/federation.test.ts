import { config, graph, connector } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Single()

describe('Federation generator', () => {
  beforeEach(() => g.clear())

  it('renders federation when enabled', async () => {
    g.type('Post', {
      id: g.id()
    })
    const cfg = config({
      schema: g,
      federation: {
        version: '2.3'
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema 


      @federation(version: "2.3")


      type Post {
        id: ID!
      }"
    `)
  })
})

describe('Federation generator with connector', () => {
  beforeEach(() => g.clear())

  it('renders federation when enabled', async () => {
    g.type('Post', {
      id: g.id()
    })

    const greenlake = connector.OpenAPI('greenlake', {
      schema:
        'https://gist.githubusercontent.com/fbjork/5168153d8fe31998014c43238be47c4e/raw/43de74de63792b30f64e165078d68e63a2c81baf/greenlake.yml'
    })

    g.datasource(greenlake)
    const cfg = config({
      schema: g,
      federation: {
        version: '2.3'
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema 

        @openapi(
          name: "greenlake"
          namespace: true
          schema: "https://gist.githubusercontent.com/fbjork/5168153d8fe31998014c43238be47c4e/raw/43de74de63792b30f64e165078d68e63a2c81baf/greenlake.yml"
        )


      @federation(version: "2.3")


      type Post {
        id: ID!
      }"
    `)
  })
})
