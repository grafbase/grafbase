/**
 * Defines introspection.
 */
export interface IntrospectionParams {
  enabled: boolean
}

export class Introspection {
  private params: IntrospectionParams

  constructor(params: IntrospectionParams) {
    this.params = params
  }

  public toString(): string {
    return `extend schema @introspection(enable: ${
      this.params.enabled ? 'true' : 'false'
    })\n\n`
  }
}
