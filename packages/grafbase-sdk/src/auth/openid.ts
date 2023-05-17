export interface OpenIDParams {
  issuer: string
}

export class OpenIDAuth {
  issuer: string

  constructor(params: OpenIDParams) {
    this.issuer = params.issuer
  }

  public toString(): string {
    return `{ type: oidc, issuer: "${this.issuer}" }`
  }
}