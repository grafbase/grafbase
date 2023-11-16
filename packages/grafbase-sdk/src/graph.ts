import { FederatedGraph, SingleGraph } from './grafbase-schema'

/**
 * A builder for a Grafbase schema definition.
 */
export const graph = {
  Federated: () => new FederatedGraph(),
  Single: () => new SingleGraph()
}
