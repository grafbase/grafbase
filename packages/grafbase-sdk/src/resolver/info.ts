/**
 * The type of the `info` argument in a Grafbase edge resolver.
 *
 * Reference: https://grafbase.com/docs/edge-gateway/resolvers#info
 *
 * @example
 *
 * import { Context, Info } from '@grafbase/sdk'
 *
 * export default async function(_parent, _args, context: Context, info: Info) {
 *   // ...
 * }
 */
export type ResolverInfo = {
  /** The name of the resolved field in the parent type. */
  fieldName: string
  /**  The fields traversed prior to the called resolver. */
  path: ResolverInfoPath
  /** The variables defined for the query. */
  variableValues: { [variableName: string]: any }
}

/** A field traversed prior to calling a resolver. Also see ResolverInfo. */
export type ResolverInfoPath = {
  // definition: engine/crates/engine/src/registry/resolvers/custom.rs and engine/crates/engine/src/query_path.rs
  /** The name of the field or the index of the value in a list. */
  key: string | number
  /** The name of the parent type of the field. */
  typename?: string
  /** The field traversed before this one. */
  prev?: ResolverInfoPath
}
