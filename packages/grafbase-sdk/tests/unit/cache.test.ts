import { config, g } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

describe('Cache generator', () => {
  beforeEach(() => g.clear())

  it('renders single type global cache', async () => {
    g.type('A', {
      b: g.int().optional()
    })

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

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @cache(rules: [
          {
            types: "Query",
            maxAge: 60
          }
        ])

      type A {
        b: Int
      }"
    `)
  })

  it('renders multi-type global cache', async () => {
    g.type('A', {
      b: g.int().optional()
    })

    const cfg = config({
      schema: g,
      cache: {
        rules: [
          {
            types: ['Query', 'Schwuery'],
            maxAge: 60,
            staleWhileRevalidate: 60
          }
        ]
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @cache(rules: [
          {
            types: ["Query", "Schwuery"],
            maxAge: 60,
            staleWhileRevalidate: 60
          }
        ])

      type A {
        b: Int
      }"
    `)
  })

  it('renders complex multi-type global cache', async () => {
    g.type('A', {
      b: g.int().optional()
    })

    const cfg = config({
      schema: g,
      cache: {
        rules: [
          {
            types: [
              { name: 'User' },
              { name: 'Address', fields: ['street', 'city'] }
            ],
            maxAge: 60,
            staleWhileRevalidate: 60
          }
        ]
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @cache(rules: [
          {
            types: [{
              name: "User"
            }, {
              name: "Address",
              fields: ["street","city"]
            }],
            maxAge: 60,
            staleWhileRevalidate: 60
          }
        ])

      type A {
        b: Int
      }"
    `)
  })
})
