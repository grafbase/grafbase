import { config, g } from '@grafbase/sdk'

g.query('sum', {
  args: {
    a: g.int(),
    b: g.int()
  },
  resolver: 'sum',
  returns: g.int()
})

export default config({
  schema: g
})
