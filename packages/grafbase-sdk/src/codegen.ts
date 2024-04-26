/**
 * Typescript resolver code generation
 */
export interface CodegenParams {
  enabled: boolean
  /**
   * A directory path where the types for resolvers should be generated. `generated` by default.
   */
  path?: string
}

export class Codegen {
  private params: CodegenParams

  constructor(params: CodegenParams) {
    this.params = params
  }

  public toString(): string {
    const enabled = `\n    enabled: ${this.params.enabled}`
    const path = this.params.path
      ? `,\n    path: ${JSON.stringify(this.params.path)}`
      : ''

    return `extend schema\n  @codegen(${enabled}${path}\n  )\n\n`
  }
}

