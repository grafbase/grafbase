import { config, g, connector } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'

describe('GraphQL connector', () => {
  beforeEach(() => g.clear())

  it('generates the minimum possible GraphQL datasource', () => {
    const contentful = connector.GraphQL({
      url: 'https://graphql.contentful.com/content/v1/spaces/{{ env.CONTENTFUL_SPACE_ID }}/environments/{{ env.CONTENTFUL_ENVIRONMENT }}'
    })

    g.datasource(contentful, { namespace: 'Contentful' })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "extend schema
        @graphql(
          name: "Contentful"
          url: "https://graphql.contentful.com/content/v1/spaces/{{ env.CONTENTFUL_SPACE_ID }}/environments/{{ env.CONTENTFUL_ENVIRONMENT }}"
        )"
    `)
  })

  it('generates the maximum possible GraphQL datasource', () => {
    const contentful = connector.GraphQL({
      url: 'https://graphql.contentful.com/content/v1/spaces/{{ env.CONTENTFUL_SPACE_ID }}/environments/{{ env.CONTENTFUL_ENVIRONMENT }}',
      headers: (headers) => {
        headers.static('Authorization', 'Bearer {{ env.STRIPE_API_KEY }}')
        headers.static('Method', 'POST')
      }
    })

    g.datasource(contentful, { namespace: 'Contentful' })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "extend schema
        @graphql(
          name: "Contentful"
          url: "https://graphql.contentful.com/content/v1/spaces/{{ env.CONTENTFUL_SPACE_ID }}/environments/{{ env.CONTENTFUL_ENVIRONMENT }}"
          headers: [
            { name: "Authorization", value: "Bearer {{ env.STRIPE_API_KEY }}" }
            { name: "Method", value: "POST" }
          ]
        )"
    `)
  })

  it('combines multiple apis into one extension', () => {
    const contentful = connector.GraphQL({
      url: 'https://graphql.contentful.com/content/v1/spaces/{{ env.CONTENTFUL_SPACE_ID }}/environments/{{ env.CONTENTFUL_ENVIRONMENT }}'
    })

    const github = connector.GraphQL({
      url: 'https://api.github.com/graphql'
    })

    g.datasource(contentful, { namespace: 'Contentful' })
    g.datasource(github, { namespace: 'GitHub' })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "extend schema
        @graphql(
          name: "Contentful"
          url: "https://graphql.contentful.com/content/v1/spaces/{{ env.CONTENTFUL_SPACE_ID }}/environments/{{ env.CONTENTFUL_ENVIRONMENT }}"
        )
        @graphql(
          name: "GitHub"
          url: "https://api.github.com/graphql"
        )"
    `)
  })
})
