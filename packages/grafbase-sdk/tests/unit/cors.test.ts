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
        allowedHeaders: ['Authorization'],
        allowedMethods: [
          'GET',
          'POST',
          'PUT',
          'DELETE',
          'HEAD',
          'OPTIONS',
          'CONNECT',
          'PATCH',
          'TRACE'
        ],
        exposedHeaders: ['Authorization'],
        allowCredentials: true,
        allowedOrigins: [new URL('https://example.com')]
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @cors(
          allowCredentials: true,
          maxAge: 88400,
          allowedHeaders: ["Authorization"],
          allowedMethods: ["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "CONNECT", "PATCH", "TRACE"],
          exposedHeaders: ["Authorization"],
          allowedOrigins: ["https://example.com/"]
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
          allowCredentials: false,
          allowedOrigins: "*"
        )

      type A {
        b: Int
      }"
    `)
  })

  it('with any allowed header', () => {
    const cfg = config({
      graph: g,
      cors: {
        allowedHeaders: '*'
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @cors(
          allowCredentials: false,
          allowedHeaders: "*"
        )

      type A {
        b: Int
      }"
    `)
  })

  it('with any allowed method', () => {
    const cfg = config({
      graph: g,
      cors: {
        allowedMethods: '*'
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @cors(
          allowCredentials: false,
          allowedMethods: "*"
        )

      type A {
        b: Int
      }"
    `)
  })

  it('with any exposed header', () => {
    const cfg = config({
      graph: g,
      cors: {
        exposedHeaders: '*'
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @cors(
          allowCredentials: false,
          exposedHeaders: "*"
        )

      type A {
        b: Int
      }"
    `)
  })

  it('with no defined settings', () => {
    const cfg = config({
      graph: g,
      cors: {}
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @cors(
          allowCredentials: false
        )

      type A {
        b: Int
      }"
    `)
  })
})
