export interface JWTParams {
  issuer: string
  secret: string
  clientId?: string
  groupsClaim?: string
}

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
