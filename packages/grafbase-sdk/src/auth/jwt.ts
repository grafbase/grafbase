/**
 * Input parameters to define a JWT auth provider. Requres `issuer` and `secret`
 * to be defined, and optionally supports the `clientId` and `groupsClaim`
 * definitions.
 *
 * `clientId` should be defined for providers that sign tokens with the same
 * `iss` value. The value of `clientId` is checked against the `aud` claim
 * inside the JWT.
 *
 * `groupsClaim` should be defined for group-based auth to use a custom claim
 * path.
 */
export interface JWTParams {
  issuer: string
  secret: string
  clientId?: string
  groupsClaim?: string
}

/**
 * Grafbase supports a symmetric JWT provider that you can use to authorize
 * requests using a JWT signed by yourself or a third-party service.
 */
export class JWTAuth {
  issuer: string
  secret: string
  clientId?: string
  groupsClaim?: string

  constructor(params: JWTParams) {
    this.issuer = params.issuer
    this.secret = params.secret
    this.clientId = params.clientId
    this.groupsClaim = params.groupsClaim
  }

  public toString(): string {
    const clientId = this.clientId ? `, clientId: "${this.clientId}"` : ''
    const groupsClaim = this.groupsClaim
      ? `, groupsClaim: "${this.groupsClaim}"`
      : ''

    return `{ type: jwt, issuer: "${this.issuer}", secret: "${this.secret}"${clientId}${groupsClaim} }`
  }
}
