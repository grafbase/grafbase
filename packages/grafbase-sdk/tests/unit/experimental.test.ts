import { config, g } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

describe('Experimental generator', () => {
  beforeEach(() => g.clear())

  it('renders experimental with kv enabled', async () => {
    const cfg = config({
      schema: g,
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
      schema: g,
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
      schema: g,
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
      schema: g,
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
      schema: g,
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

  it('doesnt render experimental', async () => {
    const cfg = config({
      schema: g,
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
      schema: g,
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
