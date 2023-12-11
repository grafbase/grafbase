import { FederatedGraph, FederatedGraphInput, Graph } from './grafbase-schema'

export interface StandaloneInput {
  subgraph: boolean
}

/**
 * A builder for a Grafbase schema definition.
 */
export const graph = {
  Federated: (input?: FederatedGraphInput) => new FederatedGraph(input),
  Standalone: (input?: StandaloneInput) => new Graph(Boolean(input?.subgraph))
}
