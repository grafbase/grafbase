import { config, graph, connector } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Single()

describe('MongoDB generator', () => {
  const mongoParams = {
    url: 'https://data.mongodb-api.com/app/data-test/endpoint/data/v1',
    apiKey: 'SOME_KEY',
    dataSource: 'data',
    database: 'tables'
  }

  beforeEach(() => g.clear())

  it('generates the minimum possible MongoDB datasource', () => {
    const mongo = connector.MongoDB('Test', mongoParams)
    g.datasource(mongo)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema 

        @mongodb(
          namespace: true
          name: "Test"
          url: "https://data.mongodb-api.com/app/data-test/endpoint/data/v1"
          apiKey: "SOME_KEY"
          dataSource: "data"
          database: "tables"
        )"
    `)
  })

  it('generates the minimum possible MongoDB datasource, namespace: false', () => {
    const mongo = connector.MongoDB('Test', mongoParams)
    g.datasource(mongo, { namespace: false })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema 

        @mongodb(
          namespace: false
          name: "Test"
          url: "https://data.mongodb-api.com/app/data-test/endpoint/data/v1"
          apiKey: "SOME_KEY"
          dataSource: "data"
          database: "tables"
        )"
    `)
  })

  it('generates a simple model', () => {
    const mongo = connector.MongoDB('Test', mongoParams)

    g.datasource(mongo)

    mongo
      .model('User', {
        id: g.id().unique().mapped('_id'),
        field: g.string()
      })
      .collection('users')

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema 

        @mongodb(
          namespace: true
          name: "Test"
          url: "https://data.mongodb-api.com/app/data-test/endpoint/data/v1"
          apiKey: "SOME_KEY"
          dataSource: "data"
          database: "tables"
        )

      type User @model(connector: "Test", collection: "users") {
        id: ID! @unique @map(name: "_id")
        field: String!
      }"
    `)
  })

  it('expects a valid identifier as a name', () => {
    expect(() => connector.MongoDB('Nest Test', mongoParams)).toThrow(
      'Given name "Nest Test" is not a valid TypeScript identifier.'
    )
  })

  it('generates a simple model with a nested type', () => {
    const mongo = connector.MongoDB('Test', mongoParams)

    g.datasource(mongo)

    const address = g.type('Address', {
      street: g.string().mapped('street_name')
    })

    mongo
      .model('User', {
        address: g.ref(address)
      })
      .collection('users')

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema 

        @mongodb(
          namespace: true
          name: "Test"
          url: "https://data.mongodb-api.com/app/data-test/endpoint/data/v1"
          apiKey: "SOME_KEY"
          dataSource: "data"
          database: "tables"
        )

      type Address {
        street: String! @map(name: "street_name")
      }

      type User @model(connector: "Test", collection: "users") {
        address: Address!
      }"
    `)
  })

  it('generates a simple model with no specified collection', () => {
    const mongo = connector.MongoDB('Test', mongoParams)

    mongo.model('User', {
      id: g.id().unique().mapped('_id'),
      field: g.string()
    })

    g.datasource(mongo)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema 

        @mongodb(
          namespace: true
          name: "Test"
          url: "https://data.mongodb-api.com/app/data-test/endpoint/data/v1"
          apiKey: "SOME_KEY"
          dataSource: "data"
          database: "tables"
        )

      type User @model(connector: "Test", collection: "User") {
        id: ID! @unique @map(name: "_id")
        field: String!
      }"
    `)
  })

  it('generates a decimal field', () => {
    const mongo = connector.MongoDB('Test', mongoParams)

    mongo
      .model('User', {
        id: g.id().unique().mapped('_id'),
        field: g.decimal()
      })
      .collection('users')

    g.datasource(mongo)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema 

        @mongodb(
          namespace: true
          name: "Test"
          url: "https://data.mongodb-api.com/app/data-test/endpoint/data/v1"
          apiKey: "SOME_KEY"
          dataSource: "data"
          database: "tables"
        )

      type User @model(connector: "Test", collection: "users") {
        id: ID! @unique @map(name: "_id")
        field: Decimal!
      }"
    `)
  })

  it('generates a bytes field', () => {
    const mongo = connector.MongoDB('Test', mongoParams)

    mongo
      .model('User', {
        id: g.id().unique().mapped('_id'),
        field: g.bytes()
      })
      .collection('users')

    g.datasource(mongo)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema 

        @mongodb(
          namespace: true
          name: "Test"
          url: "https://data.mongodb-api.com/app/data-test/endpoint/data/v1"
          apiKey: "SOME_KEY"
          dataSource: "data"
          database: "tables"
        )

      type User @model(connector: "Test", collection: "users") {
        id: ID! @unique @map(name: "_id")
        field: Bytes!
      }"
    `)
  })

  it('generates a bigint field', () => {
    const mongo = connector.MongoDB('Test', mongoParams)

    mongo
      .model('User', {
        id: g.id().unique().mapped('_id'),
        field: g.bigint()
      })
      .collection('users')

    g.datasource(mongo)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema 

        @mongodb(
          namespace: true
          name: "Test"
          url: "https://data.mongodb-api.com/app/data-test/endpoint/data/v1"
          apiKey: "SOME_KEY"
          dataSource: "data"
          database: "tables"
        )

      type User @model(connector: "Test", collection: "users") {
        id: ID! @unique @map(name: "_id")
        field: BigInt!
      }"
    `)
  })

  it('generates a model with auth', () => {
    const mongo = connector.MongoDB('Test', mongoParams)

    mongo
      .model('User', {
        id: g.id().unique().mapped('_id'),
        field: g.string()
      })
      .collection('users')
      .auth((rules) => {
        rules.private()
      })

    g.datasource(mongo)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema 

        @mongodb(
          namespace: true
          name: "Test"
          url: "https://data.mongodb-api.com/app/data-test/endpoint/data/v1"
          apiKey: "SOME_KEY"
          dataSource: "data"
          database: "tables"
        )

      type User @model(connector: "Test", collection: "users") @auth(
          rules: [
            { allow: private }
          ]) {
        id: ID! @unique @map(name: "_id")
        field: String!
      }"
    `)
  })

  it('generates a model with cache', () => {
    const mongo = connector.MongoDB('Test', mongoParams)

    mongo
      .model('User', {
        id: g.id().unique().mapped('_id'),
        field: g.string()
      })
      .collection('users')
      .cache({ maxAge: 30 })

    g.datasource(mongo)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema 

        @mongodb(
          namespace: true
          name: "Test"
          url: "https://data.mongodb-api.com/app/data-test/endpoint/data/v1"
          apiKey: "SOME_KEY"
          dataSource: "data"
          database: "tables"
        )

      type User @model(connector: "Test", collection: "users") @cache(maxAge: 30) {
        id: ID! @unique @map(name: "_id")
        field: String!
      }"
    `)
  })

  it('generates a two datasources with separate models', () => {
    const test = connector.MongoDB('Test', mongoParams)

    const another = connector.MongoDB('Another', {
      url: 'https://data.mongodb-api.com/app/data-jest/endpoint/data/v1',
      apiKey: 'OTHER_KEY',
      dataSource: 'bar',
      database: 'something'
    })

    test
      .model('User', {
        id: g.id().unique().mapped('_id')
      })
      .collection('users')

    another
      .model('Post', {
        id: g.id().unique().mapped('_id')
      })
      .collection('posts')

    g.datasource(test)
    g.datasource(another)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema 

        @mongodb(
          namespace: true
          name: "Test"
          url: "https://data.mongodb-api.com/app/data-test/endpoint/data/v1"
          apiKey: "SOME_KEY"
          dataSource: "data"
          database: "tables"
        )
        @mongodb(
          namespace: true
          name: "Another"
          url: "https://data.mongodb-api.com/app/data-jest/endpoint/data/v1"
          apiKey: "OTHER_KEY"
          dataSource: "bar"
          database: "something"
        )

      type User @model(connector: "Test", collection: "users") {
        id: ID! @unique @map(name: "_id")
      }

      type Post @model(connector: "Another", collection: "posts") {
        id: ID! @unique @map(name: "_id")
      }"
    `)
  })
})
