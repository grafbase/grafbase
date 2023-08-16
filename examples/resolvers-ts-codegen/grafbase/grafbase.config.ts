import { config, g } from '@grafbase/sdk'

const MyType = g.type('MyType', {
  total: g.int(),
  inputA: g.int(),
  inputB: g.int()
})

g.query('sum', {
  args: {
    a: g.int(),
    b: g.int()
  },
  resolver: 'sum',
  returns: g.ref(MyType)
})

export default config({
  schema: g
})
