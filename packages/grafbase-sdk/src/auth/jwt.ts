export interface JWTParams {
  issuer: string,
  secret: string,
  clientId?: string,
}

export class JWTAuth {
  issuer: string
  secret: string
  clientId?: string

  constructor(params: JWTParams) {
    this.issuer = params.issuer
    this.secret = params.secret
    this.clientId = params.clientId
  }

  public toString(): string {
    const clientId = this.clientId ? `, clientId: "${this.clientId}"` : ""

    return `{ type: jwt, issuer: "${this.issuer}", secret: "${this.secret}"${clientId} }`
  }
}