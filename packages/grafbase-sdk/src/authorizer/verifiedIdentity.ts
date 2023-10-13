/**
 * The data returned by a [Custom Authorizer](https://grafbase.com/docs/auth/providers#custom-authorizer) when it can verify the identity of the incoming request.
 *
 * @example
 * 
 * // grafbase/authorizers/myJwt.ts
 *
 * import { AuthorizerContext, VerifiedIdentity } from '@grafbase/sdk'
 *
 * export default async ({ request }: AuthorizerContext): VerifiedIdentity? {
 *   // ...
 * }
 */
export type VerifiedIdentity = {
  identity: {
    /** The identity subject (= owner). */
    sub?: string 
    /** Groups the authentified request belongs to. */
    groups?: string[]
    /** Extra, custom token claims. */
    [tokenClaim: string]: any
  } 
}
