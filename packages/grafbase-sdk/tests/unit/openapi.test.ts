import { config, g, connector } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

describe('OpenAPI generator', () => {
  beforeEach(() => g.clear())

  it('generates the minimum possible OpenAPI datasource', () => {
    const stripe = connector.OpenAPI({
      schema:
        'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json'
    })

    g.datasource(stripe)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema
        @openapi(
          schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json"
        )"
    `)
  })

  it('generates the maximum possible OpenAPI datasource', () => {
    const stripe = connector.OpenAPI({
      schema:
        'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json',
      url: 'https://api.stripe.com',
      transforms: (schema) => {
        schema.queryNaming('OPERATION_ID')
        schema.exclude('Foo.Bar', 'Bar.Foo')
        schema.exclude('Foo.*.bar')
      },
      headers: (headers) => {
        headers.static('Authorization', 'Bearer {{ env.STRIPE_API_KEY }}')
        headers.static('Method', 'POST')

        headers.introspection('foo', 'bar')

        headers.set('X-Features', 'Foo,Bar')
        headers.set('Foo', { forward: 'Bar' })
      }
    })

    g.datasource(stripe, { namespace: 'Stripe' })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema
        @openapi(
          namespace: "Stripe"
          url: "https://api.stripe.com"
          schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json"
          transforms: {
            exclude: [
              "Foo.Bar"
              "Bar.Foo"
              "Foo.*.bar"
            ]
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
    const stripe = connector.OpenAPI({
      schema:
        'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json'
    })

    const openai = connector.OpenAPI({
      schema:
        'https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml'
    })

    g.datasource(stripe, { namespace: 'Stripe' })
    g.datasource(openai, { namespace: 'OpenAI' })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema
        @openapi(
          namespace: "Stripe"
          schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json"
        )
        @openapi(
          namespace: "OpenAI"
          schema: "https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml"
        )"
    `)
  })
})
