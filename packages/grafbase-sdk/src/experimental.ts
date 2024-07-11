/**
 * Defines the experimental config.
 */
export interface ExperimentalParams {
  /** @deprecated Resolvers KV is deprecated. Please adapt your project, and contact us if you need help with the migration. */
  kv?: boolean
  /** @deprecated AI is deprecated. Please adapt your project, and contact us if you need help with the migration.  */
  ai?: boolean
  /**
   * @deprecated Codegen was stabilized. Use the `codegen` key in the config object.
   */
  codegen?: boolean
  /** Specifies the runtime to be used for the executor: `edge` or `nodejs`. */
  runtime?: 'edge' | 'nodejs'
  /** Enables partial caching of responses */
  partialCaching?: boolean
}

export class Experimental {
  private params: ExperimentalParams

  constructor(params: ExperimentalParams) {
    this.params = params
  }

  public toString(): string {
    const params = Object.keys(this.params)
      .map((key) => {
        const value = this.params[key as keyof ExperimentalParams]
        if (typeof value === 'string') {
          return `${key}: "${value.replace('"', '\\"')}"`
        } else {
          return `${key}: ${value}`
        }
      })
      .join(', ')
    return params ? `extend schema\n  @experimental(${params})\n\n` : ''
  }
}
