import { config, g } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

describe('Enum generator', () => {
  beforeEach(() => {
    g.clear()
  })

  it('generates an enum from an array of strings', () => {
    const e = g.enum('Fruits', ['Apples', 'Oranges'])

    expect(renderGraphQL(e)).toMatchInlineSnapshot(`
      "enum Fruits {
        Apples,
        Oranges
      }"
    `)
  })

  it('generates an enum from a typescript enum', () => {
    enum Fruits {
      Apples,
      Oranges
    }

    const e = g.enum('Fruits', Fruits)

    expect(renderGraphQL(e)).toMatchInlineSnapshot(`
      "enum Fruits {
        Apples,
        Oranges
      }"
    `)
  })

  it('generates an enum field', () => {
    const e = g.enum('Fruits', ['Apples', 'Oranges'])

    g.model('Basket', {
      fruitType: g.ref(e)
    })

    expect(renderGraphQL(config({ schema: g }))).toMatchInlineSnapshot(`
      "enum Fruits {
        Apples,
        Oranges
      }

      type Basket @model {
        fruitType: Fruits!
      }"
    `)
  })
})
