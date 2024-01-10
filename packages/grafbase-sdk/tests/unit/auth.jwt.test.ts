import { config, graph, auth } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Standalone()

describe('OpenID auth provider', () => {
  beforeEach(() => g.clear())

  it('renders a provider with private access', () => {
    const derp = auth.JWT({
      issuer: '{{ env.ISSUER_URL }}',
      secret: '{{ env.JWT_SECRET }}'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [derp],
        rules: (rules) => {
          rules.private()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: jwt, issuer: "{{ env.ISSUER_URL }}", secret: "{{ env.JWT_SECRET }}" }
          ]
          rules: [
            { allow: private }
          ]
        )"
    `)
  })

  it('renders a provider with custom clientId', () => {
    const derp = auth.JWT({
      issuer: '{{ env.ISSUER_URL }}',
      secret: '{{ env.JWT_SECRET }}',
      clientId: 'some-id'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [derp],
        rules: (rules) => {
          rules.private()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: jwt, issuer: "{{ env.ISSUER_URL }}", secret: "{{ env.JWT_SECRET }}", clientId: "some-id" }
          ]
          rules: [
            { allow: private }
          ]
        )"
    `)
  })

  it('renders a provider with custom groupsClaim', () => {
    const derp = auth.JWT({
      issuer: '{{ env.ISSUER_URL }}',
      secret: '{{ env.JWT_SECRET }}',
      groupsClaim: 'admin'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [derp],
        rules: (rules) => {
          rules.private()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: jwt, issuer: "{{ env.ISSUER_URL }}", secret: "{{ env.JWT_SECRET }}", groupsClaim: "admin" }
          ]
          rules: [
            { allow: private }
          ]
        )"
    `)
  })

  it('renders JWT auth v2 for federated graph', () => {
    const cfg = config({
      graph: graph.Federated(),
      auth: {
        providers: [
          auth.JWT({
            jwks: {
              url: 'https:://example.com/.well-known/jwks.json'
            }
          })
        ]
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @authz(
          providers: [
            { type: jwt, jwks: { url: "https:://example.com/.well-known/jwks.json" } }
          ]
        )
      extend schema
        @graph(type: federated)
      "
    `)
  })

  it('renders JWT auth v2 for federated graph with all options', () => {
    const cfg = config({
      graph: graph.Federated(),
      auth: {
        providers: [
          auth.JWT({
            name: 'my-jwt',
            jwks: {
              url: 'https:://example.com/.well-known/jwks.json',
              issuer: 'https://example.com',
              audience: 'me',
              pollInterval: '60s'
            },
            header: {
              name: 'Authorization',
              valuePrefix: 'Bearer '
            }
          })
        ]
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @authz(
          providers: [
            { type: jwt, jwks: { url: "https:://example.com/.well-known/jwks.json", issuer: "https://example.com", audience: "me", pollInterval: "60s" }, header: { name: "Authorization", valuePrefix: "Bearer " }, name: "my-jwt" }
          ]
        )
      extend schema
        @graph(type: federated)
      "
    `)
  })
})
