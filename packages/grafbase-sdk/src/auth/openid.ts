export interface OpenIDParams {
  issuer: string
  clientId?: string
  groupsClaim?: string
}

export class OpenIDAuth {
  issuer: string
  clientId?: string
  groupsClaim?: string

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
