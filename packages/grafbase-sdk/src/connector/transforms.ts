export type TransformsGenerator = (schema: SchemaTransforms) => void
export type SchemaTransform = ExcludeTransform | TypePrefixTransform

/**
 * An accumulator class to gather transforms for a connector that introspects
 * and generates its schema
 */
export class SchemaTransforms {
  private _excludes: string[]
  private _prefixTypes: TypePrefixTransform | null

  constructor() {
    this._excludes = []
    this._prefixTypes = null
  }

  public get transforms(): SchemaTransform[] {
    const transforms = []
    if (this._excludes.length != 0) {
      transforms.push(new ExcludeTransform(this._excludes))
    }
    if (this._prefixTypes !== null) {
      transforms.push(this._prefixTypes)
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

  /**
   * Sets the prefix that will be applied to the name of all of this connectors
   * types.
   *
   * This defaults to the name of the connector if the connector is namespaced
   *
   * @param prefix - The prefix to use
   */
  public prefixTypes(prefix: string) {
    this._prefixTypes = new TypePrefixTransform(prefix)
  }
}

/**
 * A transform that excludes types or fields from a connectors output
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

/**
 * A transform that sets the prefix to use for a connectors generated types
 */
export class TypePrefixTransform {
  private prefix: string

  constructor(prefix: string) {
    this.prefix = prefix
  }

  public toString(): string {
    return `typePrefix: "${this.prefix}"`
  }
}
