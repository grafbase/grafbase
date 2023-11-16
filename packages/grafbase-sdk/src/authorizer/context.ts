/**
 * The type of the `context` argument in a [Custom Authorizer](https://grafbase.com/docs/auth/providers#custom-authorizer).
 *
 * @example
 *
 * // grafbase/authorizers/myJwt.ts
 *
 * import { AuthorizerContext, VerifiedIdentity } from '@grafbase/sdk'
 *
 * export default async function({ request }: AuthorizerContext): VerifiedIdentity? {
 *   // ...
 * }
 */
export type AuthorizerContext = {
  /** The incoming HTTP request. */
  request: {
    headers: Record<string, string>
  }
}
