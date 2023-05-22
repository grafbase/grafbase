import { RequireExactlyOne } from "type-fest"

export type JWKSParams = {
  issuer?: string,
  jwksEndpoint?: string,
  clientId?: string
  groupsClaim?: string
}

export class JWKSAuth {
  issuer?: string
  jwksEndpoint?: string
  clientId?: string
  groupsClaim?: string

  constructor(params: RequireExactlyOne<JWKSParams, 'issuer' | 'jwksEndpoint'>) {
    this.issuer = params.issuer
    this.jwksEndpoint = params.jwksEndpoint
    this.clientId = params.clientId
    this.groupsClaim = params.groupsClaim
  }

  public toString(): string {
    const issuer = this.issuer ? `issuer: "${this.issuer}"` : ""
    const jwksEndpoint = this.jwksEndpoint ? `jwksEndpoint: "${this.jwksEndpoint}"` : ""
    const clientId = this.clientId ? `, clientId: "${this.clientId}"` : ""
    const groupsClaim = this.groupsClaim ? `, groupsClaim: "${this.groupsClaim}"` : ""

    return `{ type: jwks, ${issuer}${jwksEndpoint}${clientId}${groupsClaim} }`
  }
}