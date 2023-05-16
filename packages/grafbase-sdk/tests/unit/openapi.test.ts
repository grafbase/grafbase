import { config, g, connector } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'

describe('OpenAPI generator', () => {
  beforeEach(() => g.clear())

  it('generates the minimum possible OpenAPI datasource', () => {
    const stripe = connector.OpenAPI({
      schema:
        'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json'
    })

    g.introspect(stripe, { namespace: 'Stripe' })

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
    const stripe = connector
      .OpenAPI({
        schema:
          'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json',
        url: 'https://api.stripe.com'
      })
      .header('Authorization', 'Bearer {{ env.STRIPE_API_KEY }}')

    g.introspect(stripe, { namespace: 'Stripe' })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "extend schema
        @openapi(
          name: "Stripe"
          url: "https://api.stripe.com"
          schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json"
          headers: [
            { name: "Authorization", value: "Bearer {{ env.STRIPE_API_KEY }}"}
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

    g.introspect(stripe, { namespace: 'Stripe' })
    g.introspect(openai, { namespace: 'OpenAI' })

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
