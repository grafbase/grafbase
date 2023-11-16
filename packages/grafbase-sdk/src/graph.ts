import { GrafbaseFederationSchema, GrafbaseSchema } from './grafbase-schema'

/**
 * A builder for a Grafbase schema definition.
 */
export const graph = {
  Federation: () => new GrafbaseFederationSchema(),
  Single: () => new GrafbaseSchema()
}
