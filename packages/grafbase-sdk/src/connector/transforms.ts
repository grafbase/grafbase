export type TransformsGenerator = (schema: SchemaTransforms) => void
export type SchemaTransform = ExcludeTransform

/**
 * An accumulator class to gather transforms for a connector that introspects
 * and generates its schema
 */
export class SchemaTransforms {
  private _excludes: string[]

  constructor() {
    this._excludes = []
  }

  public get transforms(): SchemaTransform[] {
    const transforms = []
    if (this._excludes.length != 0) {
      transforms.push(new ExcludeTransform(this._excludes))
    }
    return transforms
  }

  /**
   * Excludes one or more fields from the connectors schema
   *
   * @param name - The fields to exclude in dot notation
   *               e.g. `MyType.myField`, `MyType.*.someNestedField`, `{User,Account}.email`
   */
  public exclude(...name: string[]) {
    this._excludes.push(...name)
  }
}

/**
 * Header used in connector calls.
 */
export class ExcludeTransform {
  private values: string[]

  constructor(values: string[]) {
    this.values = values
  }

  public toString(): string {
    if (this.values.length == 0) {
      return ''
    }

    const excludes = this.values
      .map((exclude) => `        "${exclude}"`)
      .join('\n')

    return `exclude: [\n${excludes}\n      ]`
  }
}
