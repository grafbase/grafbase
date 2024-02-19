import { config, graph, auth } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Standalone()

describe('OpenID auth provider', () => {
  beforeEach(() => g.clear())

  it('public access', () => {
    const cfg = config({
      schema: g,
      auth: {
        rules: (rules) => {
          rules.public()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          rules: [
            { allow: public }
          ]
        )

      "
    `)
  })

  it('public access with introspection operations', () => {
    const cfg = config({
      schema: g,
      auth: {
        rules: (rules) => {
          rules.public().introspection()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          rules: [
            { allow: public, operations: [introspection] }
          ]
        )

      "
    `)
  })

  it('public access with read and introspection operations', () => {
    const cfg = config({
      schema: g,
      auth: {
        rules: (rules) => {
          rules.public().read().introspection()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          rules: [
            { allow: public, operations: [read, introspection] }
          ]
        )

      "
    `)
  })

  it('renders a provider with private access', () => {
    const clerk = auth.OpenIDConnect({
      issuer: '{{ env.ISSUER_URL }}'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [clerk],
        rules: (rules) => {
          rules.private()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: oidc, issuer: "{{ env.ISSUER_URL }}" }
          ]
          rules: [
            { allow: private }
          ]
        )

      "
    `)
  })

  it('renders a provider with custom clientId', () => {
    const clerk = auth.OpenIDConnect({
      issuer: '{{ env.ISSUER_URL }}',
      clientId: 'some-id'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [clerk],
        rules: (rules) => {
          rules.private()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: oidc, issuer: "{{ env.ISSUER_URL }}", clientId: "some-id" }
          ]
          rules: [
            { allow: private }
          ]
        )

      "
    `)
  })

  it('renders a provider with custom groupsClaim', () => {
    const clerk = auth.OpenIDConnect({
      issuer: '{{ env.ISSUER_URL }}',
      groupsClaim: 'admin'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [clerk],
        rules: (rules) => {
          rules.private()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: oidc, issuer: "{{ env.ISSUER_URL }}", groupsClaim: "admin" }
          ]
          rules: [
            { allow: private }
          ]
        )

      "
    `)
  })

  it('renders a provider with private access for get', () => {
    const clerk = auth.OpenIDConnect({
      issuer: '{{ env.ISSUER_URL }}'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [clerk],
        rules: (rules) => {
          rules.private().get()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: oidc, issuer: "{{ env.ISSUER_URL }}" }
          ]
          rules: [
            { allow: private, operations: [get] }
          ]
        )

      "
    `)
  })

  it('renders a provider with private access for list', () => {
    const clerk = auth.OpenIDConnect({
      issuer: '{{ env.ISSUER_URL }}'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [clerk],
        rules: (rules) => {
          rules.private().list()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: oidc, issuer: "{{ env.ISSUER_URL }}" }
          ]
          rules: [
            { allow: private, operations: [list] }
          ]
        )

      "
    `)
  })

  it('renders a provider with private access for read', () => {
    const clerk = auth.OpenIDConnect({
      issuer: '{{ env.ISSUER_URL }}'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [clerk],
        rules: (rules) => {
          rules.private().read()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: oidc, issuer: "{{ env.ISSUER_URL }}" }
          ]
          rules: [
            { allow: private, operations: [read] }
          ]
        )

      "
    `)
  })

  it('renders a provider with private access for create', () => {
    const clerk = auth.OpenIDConnect({
      issuer: '{{ env.ISSUER_URL }}'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [clerk],
        rules: (rules) => {
          rules.private().create()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: oidc, issuer: "{{ env.ISSUER_URL }}" }
          ]
          rules: [
            { allow: private, operations: [create] }
          ]
        )

      "
    `)
  })

  it('renders a provider with private access for update', () => {
    const clerk = auth.OpenIDConnect({
      issuer: '{{ env.ISSUER_URL }}'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [clerk],
        rules: (rules) => {
          rules.private().update()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: oidc, issuer: "{{ env.ISSUER_URL }}" }
          ]
          rules: [
            { allow: private, operations: [update] }
          ]
        )

      "
    `)
  })

  it('renders a provider with private access for delete', () => {
    const clerk = auth.OpenIDConnect({
      issuer: '{{ env.ISSUER_URL }}'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [clerk],
        rules: (rules) => {
          rules.private().delete()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: oidc, issuer: "{{ env.ISSUER_URL }}" }
          ]
          rules: [
            { allow: private, operations: [delete] }
          ]
        )

      "
    `)
  })

  it('renders a provider with private access for get, list and read', () => {
    const clerk = auth.OpenIDConnect({
      issuer: '{{ env.ISSUER_URL }}'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [clerk],
        rules: (rules) => {
          rules.private().get().list().read()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: oidc, issuer: "{{ env.ISSUER_URL }}" }
          ]
          rules: [
            { allow: private, operations: [get, list, read] }
          ]
        )

      "
    `)
  })

  it('renders a provider with private access for read and introspection', () => {
    const clerk = auth.OpenIDConnect({
      issuer: '{{ env.ISSUER_URL }}'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [clerk],
        rules: (rules) => {
          rules.private().read().introspection()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: oidc, issuer: "{{ env.ISSUER_URL }}" }
          ]
          rules: [
            { allow: private, operations: [read, introspection] }
          ]
        )

      "
    `)
  })

  it('renders a provider with groups access', () => {
    const clerk = auth.OpenIDConnect({
      issuer: '{{ env.ISSUER_URL }}'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [clerk],
        rules: (rules) => {
          rules.groups(['backend', 'admin'])
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: oidc, issuer: "{{ env.ISSUER_URL }}" }
          ]
          rules: [
            { allow: groups, groups: ["backend", "admin"] }
          ]
        )

      "
    `)
  })

  it('renders a provider with groups access and custom operations', () => {
    const clerk = auth.OpenIDConnect({
      issuer: '{{ env.ISSUER_URL }}'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [clerk],
        rules: (rules) => {
          rules.groups(['backend', 'admin']).delete()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: oidc, issuer: "{{ env.ISSUER_URL }}" }
          ]
          rules: [
            { allow: groups, groups: ["backend", "admin"], operations: [delete] }
          ]
        )

      "
    `)
  })

  it('renders multiple rules like a champ', () => {
    const clerk = auth.OpenIDConnect({
      issuer: '{{ env.ISSUER_URL }}'
    })

    const cfg = config({
      schema: g,
      auth: {
        providers: [clerk],
        rules: (rules) => {
          rules.private().read()
          rules.groups(['backend', 'admin']).delete()
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @auth(
          providers: [
            { type: oidc, issuer: "{{ env.ISSUER_URL }}" }
          ]
          rules: [
            { allow: private, operations: [read] }
            { allow: groups, groups: ["backend", "admin"], operations: [delete] }
          ]
        )

      "
    `)
  })
})
