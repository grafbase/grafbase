export interface FederationParams {
  version: '2.3'
}

export class Federation {
  private version: string

  constructor(params: FederationParams) {
    this.version = params.version
  }

  public toString(): string {
    return `\nextend schema @federation(version: "${this.version}")\n`
  }
}
