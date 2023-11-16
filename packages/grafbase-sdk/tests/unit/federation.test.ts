import { config, graph } from '../../src/index'
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
      "
      extend schema @federation(version: "2.3")
      type Post {
        id: ID!
      }"
    `)
  })
})
