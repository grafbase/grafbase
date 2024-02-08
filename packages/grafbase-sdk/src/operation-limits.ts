/**
 * Defines operation limits.
 */
export interface OperationLimitsParams {
  aliases?: number
  complexity?: number
  depth?: number
  height?: number
  rootFields?: number
}

// FIXME: Find a way to "reflect" the keys of the interface above.
const OPERATION_LIMITS_PARAMS_KEYS: (keyof OperationLimitsParams)[] = [
  'aliases',
  'complexity',
  'depth',
  'height',
  'rootFields'
] as (keyof OperationLimitsParams)[]

export class OperationLimits {
  private params: OperationLimitsParams

  constructor(params: OperationLimitsParams) {
    this.params = params
  }

  public toString(): string {
    const parameters = OPERATION_LIMITS_PARAMS_KEYS.map((key) =>
      this.params[key] ? `${key}: ${this.params[key]}` : null
    )
      .filter((value) => value != null)
      .join(', ')
    if (parameters.length === 0) {
      return ''
    } else {
      return `extend schema\n  @operationLimits(${parameters})\n\n`
    }
  }
}
