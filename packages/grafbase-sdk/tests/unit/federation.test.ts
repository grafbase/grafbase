import { config, g } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

describe('Federation generator', () => {
  beforeEach(() => g.clear())

  it('renders federation when enabled', async () => {
    g.type('Post', {
      id: g.id()
    })
    const cfg = config({
      schema: g,
      federation: {
        version: '2.3'
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "type Post {
        id: ID!
      }
      extend schema @federation(version: "2.3")
      "
    `)
  })

  it('does not render federation when disabled', async () => {
    g.type('Post', {
      id: g.id()
    })
    const cfg = config({
      schema: g
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "type Post {
        id: ID!
      }"
    `)
  })
})
