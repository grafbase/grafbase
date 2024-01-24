import { auth, graph, config } from '@grafbase/sdk'

const g = graph.Standalone()

g.query('hello', {
  args: { name: g.string().optional() },
  returns: g.string(),
  resolver: 'hello',
})
g.query('goodbye', {
  args: { name: g.string().optional() },
  returns: g.string(),
  resolver: 'goodbye',
})

const authorizer = auth.Authorizer({
  name: 'auth',
})

export default config({
  graph: g,
  auth: {
    providers: [authorizer],
    rules: (rules) => {
      rules.groups(['backend', 'g1'])
    },
  },
})
