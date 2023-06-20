import { g, config } from '@grafbase/sdk'

const user = g.model('User', {
  name: g.string(),
  message: g
    .relation(() => message)
    .optional()
    .list()
    .optional()
})

const reactionType = g.enum('ReactionType', ['SMILE', 'SAD', 'PARTY'])

const reaction = g.model('Reaction', {
  type: g.enumRef(reactionType).optional(),
  message: g.relation(() => message),
  user: g.relation(user)
})

const message = g.model('Message', {
  body: g.string(),
  reactions: g.relation(reaction).optional().list().optional(),
  room: g.relation(() => room)
})

const room = g.model('Room', {
  title: g.string(),
  private: g.boolean().default(false),
  messages: g.relation(message).optional().list().optional()
})

export default config({
  schema: g
})
