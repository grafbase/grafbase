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

export interface FederationKeyParameters {
  resolvable?: boolean
}

const DefaultFederationParameters: FederationKeyParameters = {
  resolvable: true
}

export class FederationKey {
  private fields: string
  private parameters: FederationKeyParameters

  constructor(fields: string, parameters?: FederationKeyParameters) {
    parameters = parameters || {}

    this.fields = fields
    this.parameters = { ...DefaultFederationParameters, ...parameters }
  }

  public toString(): string {
    return `@key(fields: "${this.fields}" resolvable: ${this.parameters.resolvable})`
  }
}
