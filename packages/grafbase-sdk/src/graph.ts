import { FederatedGraph, FederatedGraphInput, Graph } from './grafbase-schema'

export interface StandaloneInput {
  subgraph?: boolean
  /**
    * Optional GraphQL SDL to add to your configuration. This lets you define parts of your configuration in GraphQL and others in TypeScript, depending on your preferences.
    *
    * For example, the two configurations are equivalent:
    *
    * ```
    * const g = graph.Standalone()
    *
    *  const Invoice = g.type('Invoice', {
    *   id: g.id(),
    *   invoiceNumber: g.string(),
    *   dueDate: g.date(),
    *   totalAmount: g.int(),
    * })
    *
    * g.query('invoiceByNumber', {
    *   args: { invoiceNumber: g.string() },
    *   returns: g.ref(Invoice).optional(),
    *   resolver: 'invoice/byNumber'
    * })
    *
    * export default config({ graph: g })
    * ```
    *
    * and
    *
    * ```
    * const g = graph.Standalone({
    *   typeDefs: `
    *     type Invoice {
    *       id: ID!
    *       invoiceNumber: String!
    *       dueDate: Date!
    *       totalAmount: Int!
    *     }
    *
    *     extend type Query {
    *       invoiceByNumber(invoiceNumber: String!): Invoice @resolver(name: "invoice/byNumber")
    *     }
    *   `
    * })
    *
    * export default config({ graph: g })
    * ```
    *
    */
  typeDefs?: string
}

/**
 * A builder for a Grafbase schema definition.
 */
export const graph = {
  Federated: (input?: FederatedGraphInput) => new FederatedGraph(input),
  Standalone: (input?: StandaloneInput) => new Graph(Boolean(input?.subgraph), input?.typeDefs)
}
