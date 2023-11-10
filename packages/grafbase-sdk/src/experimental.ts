/**
 * Defines the experimental config.
 */
export interface ExperimentalParams {
  kv?: boolean
  ai?: boolean
  codegen?: boolean
}

export class Experimental {
  private params: ExperimentalParams

  constructor(params: ExperimentalParams) {
    this.params = params
  }

  public toString(): string {
    const params = Object.keys(this.params)
      .map(key => `${key}: ${(this.params as any)[key]}`)
      .join(', ');
    return params ? `extend schema\n  @experimental(${params})\n\n` : ''
  }
}
