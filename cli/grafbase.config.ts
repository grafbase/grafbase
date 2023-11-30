import { graph, config } from '@grafbase/sdk'

let g = graph.Single()

g.model('Hello', {
  world: g.string(),
})

export default config({
  graph: g,
  federation: {
    version: '2.3',
  },
})
