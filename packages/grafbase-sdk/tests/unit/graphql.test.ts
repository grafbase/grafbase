import { config, g, connector } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

describe('GraphQL connector', () => {
  beforeEach(() => g.clear())

  it('generates the minimum possible GraphQL datasource', () => {
    const contentful = connector.GraphQL('Contentful', {
      url: 'https://graphql.contentful.com/content/v1/spaces/{{ env.CONTENTFUL_SPACE_ID }}/environments/{{ env.CONTENTFUL_ENVIRONMENT }}'
    })

    g.datasource(contentful)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema
        @graphql(
          name: "Contentful"
          namespace: true
          url: "https://graphql.contentful.com/content/v1/spaces/{{ env.CONTENTFUL_SPACE_ID }}/environments/{{ env.CONTENTFUL_ENVIRONMENT }}"
        )"
    `)
  })

  it('generates the minimum possible GraphQL datasource, namespace false', () => {
    const contentful = connector.GraphQL('Contentful', {
      url: 'https://graphql.contentful.com/content/v1/spaces/{{ env.CONTENTFUL_SPACE_ID }}/environments/{{ env.CONTENTFUL_ENVIRONMENT }}'
    })

    g.datasource(contentful, { namespace: false })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema
        @graphql(
          name: "Contentful"
          namespace: false
          url: "https://graphql.contentful.com/content/v1/spaces/{{ env.CONTENTFUL_SPACE_ID }}/environments/{{ env.CONTENTFUL_ENVIRONMENT }}"
        )"
    `)
  })

  it('generates the maximum possible GraphQL datasource', () => {
    const contentful = connector.GraphQL('Contentful', {
      url: 'https://graphql.contentful.com/content/v1/spaces/{{ env.CONTENTFUL_SPACE_ID }}/environments/{{ env.CONTENTFUL_ENVIRONMENT }}',
      headers: (headers) => {
        headers.static('Authorization', 'Bearer {{ env.STRIPE_API_KEY }}')
        headers.static('Method', 'POST')
        headers.introspection('Foo', 'BAR')

        headers.set('X-Features', 'Foo,Bar')
        headers.set('Foo', { forward: 'Bar' })
      },
      transforms: (schema) => {
        schema.exclude('Foo.Bar', 'Bar.Foo')
        schema.exclude('Foo.*.bar')

        schema.prefixTypes('AFancyPrefix')
      }
    })

    g.datasource(contentful, { namespace: true })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
"extend schema
  @graphql(
    name: "Contentful"
    namespace: true
    url: "https://graphql.contentful.com/content/v1/spaces/{{ env.CONTENTFUL_SPACE_ID }}/environments/{{ env.CONTENTFUL_ENVIRONMENT }}"
    headers: [
      { name: "Authorization", value: "Bearer {{ env.STRIPE_API_KEY }}" }
      { name: "Method", value: "POST" }
      { name: "X-Features", value: "Foo,Bar" }
      { name: "Foo", forward: "Bar" }
    ]
    introspectionHeaders: [
      { name: "Foo", value: "BAR" }
    ]
    transforms: {
      exclude: [
        "Foo.Bar"
        "Bar.Foo"
        "Foo.*.bar"
      ]
      typePrefix: "AFancyPrefix"
    }
  )"
`)
  })

  it('combines multiple apis into one extension', () => {
    const contentful = connector.GraphQL('Contentful', {
      url: 'https://graphql.contentful.com/content/v1/spaces/{{ env.CONTENTFUL_SPACE_ID }}/environments/{{ env.CONTENTFUL_ENVIRONMENT }}'
    })

    const github = connector.GraphQL('GitHug', {
      url: 'https://api.github.com/graphql'
    })

    g.datasource(contentful)
    g.datasource(github)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema
        @graphql(
          name: "Contentful"
          namespace: true
          url: "https://graphql.contentful.com/content/v1/spaces/{{ env.CONTENTFUL_SPACE_ID }}/environments/{{ env.CONTENTFUL_ENVIRONMENT }}"
        )
        @graphql(
          name: "GitHug"
          namespace: true
          url: "https://api.github.com/graphql"
        )"
    `)
  })
})
