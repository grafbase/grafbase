import { g, config } from '@grafbase/sdk'

const user = g.model('User', {
  name: g.string(),
  email: g.email().unique(),
  posts: g
    .relation(() => post)
    .optional()
    .list()
    .optional(),
  comments: g
    .relation(() => comment)
    .optional()
    .list()
    .optional()
})

const post = g.model('Post', {
  author: g.relation(user),
  title: g.string(),
  url: g.url(),
  votes: g
    .relation(() => vote)
    .optional()
    .list()
    .optional(),
  comments: g
    .relation(() => comment)
    .optional()
    .list()
    .optional()
})

const comment = g.model('Comment', {
  author: g.relation(() => user),
  post: g.relation(() => post),
  content: g.string()
})

const vote = g.model('Vote', {
  user: g.relation(user),
  post: g.relation(post)
})

export default config({
  schema: g
})
