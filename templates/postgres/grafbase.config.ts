import { graph, connector, config } from '@grafbase/sdk'

const g = graph.Standalone()

const pg = connector.Postgres('pg', { url: g.env('DATABASE_URL') })
g.datasource(pg)

export default config({
  graph: g,
  auth: {
    rules: (rules) => {
      rules.public()
    }
  }
})
