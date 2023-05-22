export interface OpenIDParams {
  issuer: string,
  clientId?: string
}

export class OpenIDAuth {
  issuer: string
  clientId?: string

  constructor(params: OpenIDParams) {
    this.issuer = params.issuer
    this.clientId = params.clientId
  }

  public toString(): string {
    const clientId = this.clientId ? `, clientId: "${this.clientId}"` : ""

    return `{ type: oidc, issuer: "${this.issuer}"${clientId} }`
  }
}