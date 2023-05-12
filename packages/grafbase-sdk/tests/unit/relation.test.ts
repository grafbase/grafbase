import { g, config } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'

describe('Relations generator', () => {
  beforeEach(() => g.clear())

  it('generates 1:1 required relations', () => {
    const user = g.model('User', {
      profile: g.relation(() => profile)
    })

    const profile = g.model('Profile', {
      user: g.relation(user)
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type User @model {
        profile: Profile!
      }

      type Profile @model {
        user: User!
      }"
    `)
  })

  it('generates 1:1 optional relations', () => {
    const user = g.model('User', {
      profile: g.relation(() => profile).optional()
    })

    const profile = g.model('Profile', {
      user: g.relation(user)
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type User @model {
        profile: Profile
      }

      type Profile @model {
        user: User!
      }"
    `)
  })

  it('generates 1:m relations', () => {
    const user = g.model('User', {
      posts: g.relation(() => post).list()
    })

    const post = g.model('Post', {
      author: g.relation(user)
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type User @model {
        posts: [Post!]!
      }

      type Post @model {
        author: User!
      }"
    `)
  })

  it('generates 1:m relations with nullable content', () => {
    const user = g.model('User', {
      posts: g.relation(() => post).optional().list()
    })

    const post = g.model('Post', {
      author: g.relation(user)
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type User @model {
        posts: [Post]!
      }

      type Post @model {
        author: User!
      }"
    `)
  })

  it('generates 1:m relations with nullable list', () => {
    const user = g.model('User', {
      posts: g.relation(() => post).list().optional()
    })

    const post = g.model('Post', {
      author: g.relation(user)
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type User @model {
        posts: [Post!]
      }

      type Post @model {
        author: User!
      }"
    `)
  })

  it('generates 1:m relations with nullable list and content', () => {
    const user = g.model('User', {
      posts: g.relation(() => post).optional().list().optional()
    })

    const post = g.model('Post', {
      author: g.relation(user)
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type User @model {
        posts: [Post]
      }

      type Post @model {
        author: User!
      }"
    `)
  })

  it('generates m:m relations', () => {
    const user = g.model('User', {
      posts: g.relation(() => post).list()
    })

    const post = g.model('Post', {
      author: g.relation(user).list()
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type User @model {
        posts: [Post!]!
      }

      type Post @model {
        author: [User!]!
      }"
    `)
  })

  it('generates self-relations', () => {
    const human = g.model('Human', {
      children: g.relation(() => human).list(),
      parent: g.relation(() => human).optional()
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type Human @model {
        children: [Human!]!
        parent: Human
      }"
    `)
  })

  it('generates named relations', () => {
    const address = g.model('Address', {
      line1: g.string()
    })

    g.model('Order', {
      billingAddress: g.relation(address).name('billing'),
      shippingAddress: g.relation(address).name('shipping')
    })

    expect(config({ schema: g }).toString()).toMatchInlineSnapshot(`
      "type Address @model {
        line1: String!
      }

      type Order @model {
        billingAddress: Address! @relation(name: billing)
        shippingAddress: Address! @relation(name: shipping)
      }"
    `)
  })
})
