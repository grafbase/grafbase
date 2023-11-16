import { type ResolverContext } from './context'
import { type ResolverInfo } from './info'

/**
 * The type of a [resolver function](https://grafbase.com/docs/edge-gateway/resolvers).
 *
 * This is a generic type because different resolvers have different `parent` and `args` arguments, as well as different return types, depending on the schema.
 *
 * This type is better used through the generated resolver signatures, rather than directly.
 *
 * @example
 *
 * import { ResolverFn } from '@grafbase/sdk'
 *
 * const myResolver: ResolverFn<{ id: string }, { shout: boolean }, string> => (parent, args, _ctx, _info) => {
 *   return `parent id: ${parent.id}${args.shout ? '!' : ''}`
 * }
 *
 */
export type ResolverFn<Parent, Args, Return> =
  | ((
      parent: Parent,
      args: Args,
      context: ResolverContext,
      info: ResolverInfo
    ) => Return)
  | ((
      parent: Parent,
      args: Args,
      context: ResolverContext,
      pageInfo: ResolverInfo
    ) => Promise<Return>)
