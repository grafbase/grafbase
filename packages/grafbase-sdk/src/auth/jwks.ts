import { RequireAtLeastOne } from 'type-fest'

/**
 * Input parameters to define a JWKS auth provider.
 */
export type JWKSParams = {
  issuer?: string
  jwksEndpoint?: string
  clientId?: string
  groupsClaim?: string
}

export class JWKSAuth {
  issuer?: string
  jwksEndpoint?: string
  clientId?: string
  groupsClaim?: string

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
