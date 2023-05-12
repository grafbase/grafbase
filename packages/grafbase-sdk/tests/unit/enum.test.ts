import { config, g } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'

describe('Enum generator', () => {
  beforeEach(() => {
    g.clear()
  })

  it('generates an enum from an array of strings', () => {
    const e = g.enumType('Fruits', ['Apples', 'Oranges'])

    expect(e.toString()).toMatchInlineSnapshot(`
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

    const e = g.enumType('Fruits', Fruits)

    expect(e.toString()).toMatchInlineSnapshot(`
      "enum Fruits {
        Apples,
        Oranges
      }"
    `)
  })

  it('generates an enum field', () => {
    const e = g.enumType('Fruits', ['Apples', 'Oranges'])

    g.model('Basket', {
      fruitType: g.enum(e)
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
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
