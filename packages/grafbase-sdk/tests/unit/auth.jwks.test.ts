import { config, graph, auth } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Standalone()

describe('OpenID auth provider', () => {
  beforeEach(() => g.clear())

  it('renders a provider with private access', () => {
    const derp = auth.JWKS({
      issuer: '{{ env.ISSUER_URL }}'
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
            { type: jwks, issuer: "{{ env.ISSUER_URL }}" }
          ]
          rules: [
            { allow: private }
          ]
        )

      "
    `)
  })

  it('renders a provider with jwksEndpoint', () => {
    const derp = auth.JWKS({
      jwksEndpoint: '{{ env.JWKS_ENDPOINT }}'
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
            { type: jwks, jwksEndpoint: "{{ env.JWKS_ENDPOINT }}" }
          ]
          rules: [
            { allow: private }
          ]
        )

      "
    `)
  })

  it('renders a provider with issuer and jwksEndpoint', () => {
    const derp = auth.JWKS({
      jwksEndpoint: '{{ env.JWKS_ENDPOINT }}',
      issuer: '{{ env.ISSUER }}'
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
            { type: jwks, issuer: "{{ env.ISSUER }}", jwksEndpoint: "{{ env.JWKS_ENDPOINT }}" }
          ]
          rules: [
            { allow: private }
          ]
        )

      "
    `)
  })

  it('renders a provider with custom clientId', () => {
    const derp = auth.JWKS({
      issuer: '{{ env.ISSUER_URL }}',
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
            { type: jwks, issuer: "{{ env.ISSUER_URL }}", clientId: "some-id" }
          ]
          rules: [
            { allow: private }
          ]
        )

      "
    `)
  })

  it('renders a provider with custom groupsClaim', () => {
    const derp = auth.JWKS({
      issuer: '{{ env.ISSUER_URL }}',
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
            { type: jwks, issuer: "{{ env.ISSUER_URL }}", groupsClaim: "admin" }
          ]
          rules: [
            { allow: private }
          ]
        )

      "
    `)
  })
})
