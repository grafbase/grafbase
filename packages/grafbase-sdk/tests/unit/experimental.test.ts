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

  it('renders experimental with kv disabled', async () => {
    const cfg = config({
      schema: g,
      experimental: {
        kv: false
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
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @experimental(kv: false)
      
      "
    `)

    expect(renderGraphQL(cfg2)).toMatchInlineSnapshot(`
      "extend schema
        @cache(rules: [
          {
            types: "Query",
            maxAge: 60
          }
        ])
      
      "
    `)
  })
})
