import { config, graph, connector } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Standalone()

describe('OpenAPI generator', () => {
  beforeEach(() => g.clear())

  it('generates the minimum possible Postgres datasource', () => {
    const postgres = connector.Postgres('Postgres', {
      url: 'postgres://postgres:grafbase@localhost:5432/postgres'
    })

    g.datasource(postgres)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema
        @postgres(
          name: "Postgres"
          url: "postgres://postgres:grafbase@localhost:5432/postgres"
          namespace: true
        )"
    `)
  })

  it('generates a Postgres datasource with negative namespace', () => {
    const postgres = connector.Postgres('Postgres', {
      url: 'postgres://postgres:grafbase@localhost:5432/postgres'
    })

    g.datasource(postgres, { namespace: false })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema
        @postgres(
          name: "Postgres"
          url: "postgres://postgres:grafbase@localhost:5432/postgres"
          namespace: false
        )"
    `)
  })

  it('generates a Postgres datasource with positive namespace', () => {
    const postgres = connector.Postgres('Postgres', {
      url: 'postgres://postgres:grafbase@localhost:5432/postgres'
    })

    g.datasource(postgres, { namespace: true })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema
        @postgres(
          name: "Postgres"
          url: "postgres://postgres:grafbase@localhost:5432/postgres"
          namespace: true
        )"
    `)
  })
})
