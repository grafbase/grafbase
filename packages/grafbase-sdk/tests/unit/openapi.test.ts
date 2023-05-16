import { config, g, connector } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'

describe('OpenAPI generator', () => {
  beforeEach(() => g.clear())

  it('generates the minimum possible OpenAPI datasource', () => {
    const stripe = connector.OpenAPI({
      schema:
        'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json'
    })

    g.datasource(stripe, { namespace: 'Stripe' })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "extend schema
        @openapi(
          name: "Stripe"
          schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json"
        ) {
        query: Query
      }"
    `)
  })

  it('generates the maximum possible OpenAPI datasource', () => {
    const stripe = connector.OpenAPI({
      schema:
        'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json',
      url: 'https://api.stripe.com',
      headers: (headers) => {
        headers.static('Authorization', 'Bearer {{ env.STRIPE_API_KEY }}')
        headers.static('Method', 'POST')

        headers.introspection('foo', 'bar')
      }
    })

    g.datasource(stripe, { namespace: 'Stripe' })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "extend schema
        @openapi(
          name: "Stripe"
          url: "https://api.stripe.com"
          schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json"
          headers: [
            { name: "Authorization", value: "Bearer {{ env.STRIPE_API_KEY }}" }
            { name: "Method", value: "POST" }
          ]
          introspectionHeaders: [
            { name: "foo", value: "bar" }
          ]
        ) {
        query: Query
      }"
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

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "extend schema
        @openapi(
          name: "Stripe"
          schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json"
        )
        @openapi(
          name: "OpenAI"
          schema: "https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml"
        ) {
        query: Query
      }"
    `)
  })
})
