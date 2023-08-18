import { config, g, connector } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

describe('MongoDB generator', () => {
  const mongoParams = {
    name: 'Test',
    url: 'https://data.mongodb-api.com/app/data-test/endpoint/data/v1',
    apiKey: 'SOME_KEY',
    dataSource: 'data',
    database: 'tables'
  }

  beforeEach(() => g.clear())

  it('generates the minimum possible MongoDB datasource', () => {
    const mongo = connector.MongoDB(mongoParams)
    g.datasource(mongo)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema
        @mongodb(
          name: "Test"
          url: "https://data.mongodb-api.com/app/data-test/endpoint/data/v1"
          apiKey: "SOME_KEY"
          dataSource: "data"
          database: "tables"
        )"
    `)
  })

  it('generates a simple model', () => {
    const mongo = connector.MongoDB(mongoParams)

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

  it('generates a simple model with no specified collection', () => {
    const mongo = connector.MongoDB(mongoParams)

    mongo.model('User', {
      id: g.id().unique().mapped('_id'),
      field: g.string()
    })

    g.datasource(mongo)

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "extend schema
        @mongodb(
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
    const mongo = connector.MongoDB(mongoParams)

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
    const mongo = connector.MongoDB(mongoParams)

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
    const mongo = connector.MongoDB(mongoParams)

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
    const mongo = connector.MongoDB(mongoParams)

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
    const mongo = connector.MongoDB(mongoParams)

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
    const test = connector.MongoDB(mongoParams)

    const another = connector.MongoDB({
      name: 'Another',
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
          name: "Test"
          url: "https://data.mongodb-api.com/app/data-test/endpoint/data/v1"
          apiKey: "SOME_KEY"
          dataSource: "data"
          database: "tables"
        )
        @mongodb(
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
