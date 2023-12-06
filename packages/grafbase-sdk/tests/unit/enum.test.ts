import { config, graph } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Standalone()

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

  it('generates an enum field', () => {
    const e = g.enum('Fruits', ['Apples', 'Oranges'])

    g.model('Basket', {
      fruitType: g.enumRef(e)
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

  it('generates an enum field with a default', () => {
    const e = g.enum('Fruits', ['Apples', 'Oranges'])

    g.model('Basket', {
      fruitType: g.enumRef(e).default('Oranges')
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "enum Fruits {
        Apples,
        Oranges
      }

      type Basket @model {
        fruitType: Fruits! @default(value: Oranges)
      }"
    `)
  })

  it('prevents using of whitespaced identifier as the name', () => {
    expect(() => g.enum('white space', ['Foo', 'Bar'])).toThrow(
      'Given name "white space" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of number-prefixed identifier as the name', () => {
    expect(() => g.enum('0User', ['Foo', 'Bar'])).toThrow(
      'Given name "0User" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of weird characters identifier as the name', () => {
    expect(() => g.enum('!@#$%^&*()+|~`=-', ['Foo', 'Bar'])).toThrow(
      'Given name "!@#$%^&*()+|~`=-" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of whitespaced identifier as a variant name', () => {
    expect(() => g.enum('A', ['white space'])).toThrow(
      'Given name "white space" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of number-prefixed identifier as a variant name', () => {
    expect(() => g.enum('A', ['0User'])).toThrow(
      'Given name "0User" is not a valid TypeScript identifier.'
    )
  })

  it('prevents using of weird characters identifier as a variant name', () => {
    expect(() => g.enum('A', ['!@#$%^&*()+|~`=-'])).toThrow(
      'Given name "!@#$%^&*()+|~`=-" is not a valid TypeScript identifier.'
    )
  })
})
