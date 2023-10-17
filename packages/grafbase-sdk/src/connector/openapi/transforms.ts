import { SchemaTransforms, SchemaTransform } from '../transforms'

export type OpenApiTransformsGenerator = (schema: OpenApiTransforms) => void
export type OpenApiTransform = SchemaTransform | QueryNamingTransform

export type OpenApiQueryNamingStrategy = 'OPERATION_ID' | 'SCHEMA_NAME'

/**
 * An accumulator class to gather transforms for an OpenAPI connector
 */
export class OpenApiTransforms {
  private _schemaTransforms: SchemaTransforms
  private _queryNaming: OpenApiQueryNamingStrategy | null

  constructor() {
    this._schemaTransforms = new SchemaTransforms()
    this._queryNaming = null
  }

  public get transforms(): OpenApiTransform[] {
    const transforms: OpenApiTransform[] = this._schemaTransforms.transforms
    if (this._queryNaming != null) {
      transforms.push(new QueryNamingTransform(this._queryNaming))
    }
    return transforms
  }

  /**
   * Sets the query naming strategy for this OpenAPI connector
   */
  public queryNaming(strategy: OpenApiQueryNamingStrategy) {
    this._queryNaming = strategy
  }

  /**
   * Excludes one or more fields from the connectors schema
   *
   * @param name - The fields to exclude in dot notation
   *               e.g. `MyType.myField`, `MyType.*.someNestedField`, `{User,Account}.email`
   */
  public exclude(...name: string[]) {
    this._schemaTransforms.exclude(...name)
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
    this._schemaTransforms.prefixTypes(prefix)
  }
}

/**
 * Header used in connector calls.
 */
export class QueryNamingTransform {
  private value: OpenApiQueryNamingStrategy

  constructor(value: OpenApiQueryNamingStrategy) {
    this.value = value
  }

  public toString(): string {
    return `queryNaming: ${this.value}`
  }
}
