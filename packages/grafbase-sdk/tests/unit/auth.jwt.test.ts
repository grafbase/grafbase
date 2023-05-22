import { config, g, auth } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'

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

    expect(cfg.toString()).toMatchInlineSnapshot(`
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

    expect(cfg.toString()).toMatchInlineSnapshot(`
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
})
