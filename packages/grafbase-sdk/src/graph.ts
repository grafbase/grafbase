import { FederatedGraph, Graph } from './grafbase-schema'

export interface StandaloneInput {
  subgraph: boolean
}

/**
 * A builder for a Grafbase schema definition.
 */
export const graph = {
  Federated: () => new FederatedGraph(),
  Standalone: (input?: StandaloneInput) => new Graph(Boolean(input?.subgraph))
}
