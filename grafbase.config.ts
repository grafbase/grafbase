import { auth, config, graph } from '@grafbase/sdk'

const g = graph.Standalone()

const authorizer = auth.Authorizer({
  name: 'my-authorizer-function'
})

g.query('hello', {
  args: { name: g.string().optional() },
  returns: g.string(),
  resolver: 'hello'
})

export default config({
  graph: g,
  auth: {
    providers: [authorizer],
    rules: () => {}
  }
})
