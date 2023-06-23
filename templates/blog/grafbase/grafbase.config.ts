import { g, config } from '@grafbase/sdk'

const comment = g.model('Comment', {
  content: g.string(),
  user: g.relation(() => user).optional(),
  post: g.relation(() => post)
})

const post = g.model('Post', {
  slug: g.string().unique(),
  title: g.string(),
  content: g.string().optional(),
  user: g.relation(() => user),
  comments: g.relation(comment).optional().list().optional()
})

const user = g.model('User', {
  email: g.email().unique(),
  name: g.string(),
  posts: g.relation(post).optional().list().optional()
})

export default config({
  schema: g
})
