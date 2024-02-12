import { graph, config } from '@grafbase/sdk'

const g = graph.Federated()

export default config({
  graph: g,
  introspection: true,
})
