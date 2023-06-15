import { g, auth, config } from '@grafbase/sdk'

// Welcome to Grafbase!
// Define your data models, integrate auth, permission rules, custom resolvers, search, and more with Grafbase.
// Integrate Auth
// https://grafbase.com/docs/auth
//
// const authProvider = auth.OpenIDConnect({
//   issuer: process.env.ISSUER_URL ?? ''
// })
//
// Define Data Models
// https://grafbase.com/docs/database

const post = g.model('Post', {
  title: g.string(),
  slug: g.string().unique(),
  content: g.string().optional(),
  publishedAt: g.datetime().optional(),
  comments: g.relation(() => comment).optional().list().optional(),
  likes: g.int().default(0),
  tags: g.string().optional().list().length({ max: 5 }),
  author: g.relation(() => user).optional()
}).search()

const comment = g.model('Comment', {
  post: g.relation(post),
  body: g.string(),
  likes: g.int().default(0),
  author: g.relation(() => user).optional()
})

const user = g.model('User', {
  name: g.string(),
  email: g.email().optional(),
  posts: g.relation(post).optional().list(),
  comments: g.relation(comment).optional().list()

  // Extend models with resolvers
  // https://grafbase.com/docs/edge-gateway/resolvers
  // gravatar: g.url().resolver('user/gravatar')
})

export default config({
  schema: g
  // Integrate Auth
  // https://grafbase.com/docs/auth
  // auth: {
  //   providers: [authProvider],
  //   rules: (rules) => {
  //     rules.private()
  //   }
  // }
})
