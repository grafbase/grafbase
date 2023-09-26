/**
 * Defines the experimental config.
 */
export interface ExperimentalParams {
  kv?: boolean
}

export class Experimental {
  private params: ExperimentalParams

  constructor(params: ExperimentalParams) {
    this.params = params
  }

  public toString(): string {
    return `extend schema\n  @experimental(kv: ${this.params.kv})\n\n`
  }
}
