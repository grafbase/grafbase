import { config, graph, auth } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Standalone()

describe('Codegen settings', () => {
  beforeEach(() => {
    g.clear()
  })

  it('with all settings', () => {
    const cfg = config({
      graph: g,
      codegen: {
        enabled: false,
        path: 'test/my/path'
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @codegen(
          enabled: false,
          path: "test/my/path"
        )

      "
    `)
  })

  it('with just enabled', () => {
    const cfg = config({
      graph: g,
      codegen: { enabled: true }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @codegen(
          enabled: true
        )

      "
    `)
  })
})
