import { config, graph } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Standalone()

describe('Experimental generator', () => {
  beforeEach(() => g.clear())

  it('renders experimental with kv enabled', async () => {
    const cfg = config({
      graph: g,
      experimental: {
        kv: true
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @experimental(kv: true)
      
      "
    `)
  })

  it('renders experimental with ai enabled', async () => {
    const cfg = config({
      graph: g,
      experimental: {
        ai: true
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @experimental(ai: true)
      
      "
    `)
  })

  it('renders experimental with ai and kv enabled', async () => {
    const cfg = config({
      graph: g,
      experimental: {
        ai: true,
        kv: true
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @experimental(ai: true, kv: true)
      
      "
    `)
  })

  it('renders experimental with kv disabled', async () => {
    const cfg = config({
      graph: g,
      experimental: {
        kv: false
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @experimental(kv: false)
      
      "
    `)
  })

  it('renders experimental with ai disabled', async () => {
    const cfg = config({
      graph: g,
      experimental: {
        ai: false
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @experimental(ai: false)
      
      "
    `)
  })

  it('renders experimental with the `runtime` property', async () => {
    const cfg = config({
      graph: g,
      experimental: {
        runtime: 'nodejs'
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @experimental(runtime: "nodejs")
      
      "
    `)
  })

  it('doesnt render experimental', async () => {
    const cfg = config({
      graph: g,
      cache: {
        rules: [
          {
            types: 'Query',
            maxAge: 60
          }
        ]
      }
    })

    const cfg2 = config({
      graph: g,
      cache: {
        rules: [
          {
            types: 'Query',
            maxAge: 60
          }
        ]
      },
      experimental: {}
    })

    const expected = `
      "extend schema
        @cache(rules: [
          {
            types: "Query",
            maxAge: 60
          }
        ])
      
      "
    `
    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(expected)
    expect(renderGraphQL(cfg2)).toMatchInlineSnapshot(expected)
  })
})
