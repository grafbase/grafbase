/**
 * Input parameters to define an OpenID auth provider. Requires `issuer`
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
export interface OpenIDParams {
  issuer: string
  clientId?: string
  groupsClaim?: string
}

export class OpenIDAuth {
  private issuer: string
  private clientId?: string
  private groupsClaim?: string

  constructor(params: OpenIDParams) {
    this.issuer = params.issuer
    this.clientId = params.clientId
    this.groupsClaim = params.groupsClaim
  }

  public toString(): string {
    const clientId = this.clientId ? `, clientId: "${this.clientId}"` : ''
    const groupsClaim = this.groupsClaim
      ? `, groupsClaim: "${this.groupsClaim}"`
      : ''

    return `{ type: oidc, issuer: "${this.issuer}"${clientId}${groupsClaim} }`
  }
}
