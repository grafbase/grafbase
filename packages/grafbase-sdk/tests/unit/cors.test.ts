import { config, graph, auth } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Standalone()

describe('CORS settings', () => {
  beforeEach(() => {
    g.clear()

    g.type('A', {
      b: g.int().optional()
    })
  })

  it('with all settings', () => {
    const cfg = config({
      graph: g,
      cors: {
        maxAge: 88400,
        allowedOrigins: [new URL('https://example.com')]
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @cors(
          allowedOrigins: ["https://example.com/"],
          maxAge: 88400
        )

      type A {
        b: Int
      }"
    `)
  })

  it('with any allowed origin', () => {
    const cfg = config({
      graph: g,
      cors: {
        allowedOrigins: '*'
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @cors(
          allowedOrigins: "*"
        )

      type A {
        b: Int
      }"
    `)
  })
})
