export interface JWTParams {
  issuer: string,
  secret: string,
}

export class JWTAuth {
  issuer: string
  secret: string

  constructor(params: JWTParams) {
    this.issuer = params.issuer
    this.secret = params.secret
  }

  public toString(): string {
    return `{ type: jwt, issuer: "${this.issuer}", secret: "${this.secret}" }`
  }
}