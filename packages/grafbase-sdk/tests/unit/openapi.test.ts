import { config, g, connector } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

describe('OpenAPI generator', () => {
  beforeEach(() => g.clear())

  it('generates the minimum possible OpenAPI datasource', () => {
    const stripe = connector.OpenAPI('Stripe', {
      schema:
        'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json'
    })

    g.datasource(stripe)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema
        @openapi(
          name: "Stripe"
          namespace: true
          schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json"
        )"
    `)
  })

  it('generates the minimum possible OpenAPI datasource, namespace false', () => {
    const stripe = connector.OpenAPI('Stripe', {
      schema:
        'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json'
    })

    g.datasource(stripe, { namespace: false })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema
        @openapi(
          name: "Stripe"
          namespace: false
          schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json"
        )"
    `)
  })

  it('generates the maximum possible OpenAPI datasource', () => {
    const stripe = connector.OpenAPI('Stripe', {
      schema:
        'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json',
      url: 'https://api.stripe.com',
      transforms: (schema) => {
        schema.queryNaming('OPERATION_ID')
        schema.exclude('Foo.Bar', 'Bar.Foo')
        schema.exclude('Foo.*.bar')
        schema.prefixTypes('AFancyPrefix')
      },
      headers: (headers) => {
        headers.static('Authorization', 'Bearer {{ env.STRIPE_API_KEY }}')
        headers.static('Method', 'POST')

        headers.introspection('foo', 'bar')

        headers.set('X-Features', 'Foo,Bar')
        headers.set('Foo', { forward: 'Bar' })
      }
    })

    g.datasource(stripe)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
"extend schema
  @openapi(
    name: "Stripe"
    namespace: true
    url: "https://api.stripe.com"
    schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json"
    transforms: {
      exclude: [
        "Foo.Bar"
        "Bar.Foo"
        "Foo.*.bar"
      ]
      typePrefix: "AFancyPrefix"
      queryNaming: OPERATION_ID
    }
    headers: [
      { name: "Authorization", value: "Bearer {{ env.STRIPE_API_KEY }}" }
      { name: "Method", value: "POST" }
      { name: "X-Features", value: "Foo,Bar" }
      { name: "Foo", forward: "Bar" }
    ]
    introspectionHeaders: [
      { name: "foo", value: "bar" }
    ]
  )"
`)
  })

  it('combines multiple apis into one extension', () => {
    const stripe = connector.OpenAPI('Stripe', {
      schema:
        'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json'
    })

    const openai = connector.OpenAPI('OpenAI', {
      schema:
        'https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml'
    })

    g.datasource(stripe)
    g.datasource(openai)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema
        @openapi(
          name: "Stripe"
          namespace: true
          schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json"
        )
        @openapi(
          name: "OpenAI"
          namespace: true
          schema: "https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml"
        )"
    `)
  })
})
