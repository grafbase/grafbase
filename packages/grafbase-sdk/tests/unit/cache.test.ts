import { config, graph } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Single()

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

  it('renders type invalidation strategy', async () => {
    g.type('A', {
      b: g.int().optional()
    })

    const cfg = config({
      schema: g,
      cache: {
        rules: [
          {
            types: 'Query',
            maxAge: 60,
            mutationInvalidation: 'type'
          }
        ]
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @cache(rules: [
          {
            types: "Query",
            maxAge: 60,
            mutationInvalidation: type
          }
        ])

      type A {
        b: Int
      }"
    `)
  })

  it('renders entity invalidation strategy', async () => {
    g.type('A', {
      b: g.int().optional()
    })

    const cfg = config({
      schema: g,
      cache: {
        rules: [
          {
            types: 'Query',
            maxAge: 60,
            mutationInvalidation: 'entity'
          }
        ]
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @cache(rules: [
          {
            types: "Query",
            maxAge: 60,
            mutationInvalidation: entity
          }
        ])

      type A {
        b: Int
      }"
    `)
  })

  it('renders list invalidation strategy', async () => {
    g.type('A', {
      b: g.int().optional()
    })

    const cfg = config({
      schema: g,
      cache: {
        rules: [
          {
            types: 'Query',
            maxAge: 60,
            mutationInvalidation: 'list'
          }
        ]
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @cache(rules: [
          {
            types: "Query",
            maxAge: 60,
            mutationInvalidation: list
          }
        ])

      type A {
        b: Int
      }"
    `)
  })

  it('renders custom field invalidation strategy', async () => {
    g.type('A', {
      b: g.int().optional()
    })

    const cfg = config({
      schema: g,
      cache: {
        rules: [
          {
            types: 'Query',
            maxAge: 60,
            mutationInvalidation: { field: 'name' }
          }
        ]
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @cache(rules: [
          {
            types: "Query",
            maxAge: 60,
            mutationInvalidation: { field: "name" }
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
  it('renders global cache rule with access scopes', async () => {
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
            scopes: ['apikey', { claim: 'my_claim' }]
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
            scopes: [apikey, { claim: "my_claim" }]
          }
        ])

      type A {
        b: Int
      }"
    `)
  })
})
