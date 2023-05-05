import { config, g } from '../../src/index'
import { describe, expect, it } from '@jest/globals'

describe('Enum generator', () => {
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

    const m = g.model('Basket', {
      fruitType: g.enum(e)
    })

    const cfg = config().schema({
      models: [m],
      enums: [e]
    })

    expect(cfg.toString()).toMatchInlineSnapshot(`
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
