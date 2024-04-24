import { config, graph } from '../../src/index'
import { describe, expect, it } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Standalone({ subgraph: true })

describe('graph.Standalone() typeDef argument', () => {
  it('renders the content of typeDefs at the end', () => {
    const g = graph.Standalone({
      typeDefs: /* GraphQL */ `
        type Invoice {
          id: ID!
          invoiceNumber: String!
          dueDate: Date!
          totalAmount: Int!
        }

        extend type Query {
          invoiceByNumber(invoiceNumber: String!): Invoice @resolver(name: "invoice/byNumber")
        }
      `
    })

    const cfg = config({ graph: g })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "
              type Invoice {
                id: ID!
                invoiceNumber: String!
                dueDate: Date!
                totalAmount: Int!
              }

              extend type Query {
                invoiceByNumber(invoiceNumber: String!): Invoice @resolver(name: "invoice/byNumber")
              }
            "
    `)
  })
})
