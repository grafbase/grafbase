import { g } from '@grafbase/sdk'

g.query('hello', {
  args: { name: g.string().optional() },
  returns: g.string(),
  resolver: 'hello',
})
