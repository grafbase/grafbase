import { RequireAtLeastOne } from 'type-fest'

/**
 * Input parameters to define a JWKS auth provider. Requires `issuer` or `jwksEndpoint`
 * to be defined. If the provider does not add the `iss` claim, a `jwksEndpoint` should
 * be added including `/.well-known/jwks.json`. If both `issuer` and `jwksEndpoint` is
 * provided, `issuer` is used for claim verification and `jwksEndpoint` to fetch the keys.
 *
 * `clientId` should be defined for providers that sign tokens with the same
 * `iss` value. The value of `clientId` is checked against the `aud` claim
 * inside the JWT.
 *
 * `groupsClaim` should be defined for group-based auth to use a custom claim
 * path.
 */
export type JWKSParams = {
  issuer?: string
  jwksEndpoint?: string
  clientId?: string
  groupsClaim?: string
}

export class JWKSAuth {
  private issuer?: string
  private jwksEndpoint?: string
  private clientId?: string
  private groupsClaim?: string

  constructor(
    params: RequireAtLeastOne<JWKSParams, 'issuer' | 'jwksEndpoint'>
  ) {
    this.issuer = params.issuer
    this.jwksEndpoint = params.jwksEndpoint
    this.clientId = params.clientId
    this.groupsClaim = params.groupsClaim
  }

  public toString(): string {
    const issuer = this.issuer ? `issuer: "${this.issuer}"` : ''

    var jwksEndpoint = ''
    if (!this.issuer && this.jwksEndpoint) {
      jwksEndpoint = `jwksEndpoint: "${this.jwksEndpoint}"`
    } else if (this.jwksEndpoint) {
      jwksEndpoint = `, jwksEndpoint: "${this.jwksEndpoint}"`
    }

    const clientId = this.clientId ? `, clientId: "${this.clientId}"` : ''
    const groupsClaim = this.groupsClaim
      ? `, groupsClaim: "${this.groupsClaim}"`
      : ''

    return `{ type: jwks, ${issuer}${jwksEndpoint}${clientId}${groupsClaim} }`
  }
}
