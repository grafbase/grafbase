import { g, config } from '@grafbase/sdk'

const user = g.model('User', {
  username: g.string().unique(),
  email: g.email().unique(),
  url: g.url().optional(),
  avatar: g.url().optional(),
  likes: g
    .relation(() => like)
    .optional()
    .list()
    .optional(),
  tweets: g
    .relation(() => tweet)
    .optional()
    .list()
    .optional()
})

const tweet = g.model('Tweet', {
  text: g.string(),
  user: g.relation(user),
  likes: g
    .relation(() => like)
    .optional()
    .list()
    .optional(),
  replies: g
    .relation(() => tweet)
    .optional()
    .list()
    .optional(),
  media: g
    .relation(() => media)
    .optional()
    .list()
    .optional()
})

const like = g.model('Like', {
  tweet: g.relation(tweet),
  user: g.relation(user)
})

const mediaType = g.enum('MediaType', ['IMAGE', 'VIDEO'])

const media = g.model('Media', {
  url: g.url().optional(),
  type: g.enumRef(mediaType).optional()
})

export default config({
  schema: g
})
