import { config, connector, graph } from '@grafbase/sdk'

const g = graph.Standalone()

g.query('hello', {
  args: { name: g.string().optional() },
  returns: g.string(),
  resolver: 'hello',
})

export default config({
  graph: g,
})
