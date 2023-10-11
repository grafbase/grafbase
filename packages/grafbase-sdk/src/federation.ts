export interface FederationParams {
  version: string
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
