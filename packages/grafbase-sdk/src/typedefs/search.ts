import { AuthRuleF } from '../auth'
import { ListDefinition } from './list'
import { AuthDefinition } from './auth'
import { CacheDefinition, FieldCacheParams, FieldLevelCache } from './cache'
import { LengthLimitedStringDefinition } from './length-limited-string'
import { ScalarDefinition } from './scalar'
import { UniqueDefinition } from './unique'
import { EnumDefinition } from './enum'

/**
 * A list of field types that can hold a `@search` attribute.
 */
export type Searchable =
  | ScalarDefinition
  | ListDefinition
  | UniqueDefinition
  | LengthLimitedStringDefinition
  | CacheDefinition
  | AuthDefinition
  | EnumDefinition<any, any>

export class SearchDefinition {
  private field: Searchable

  constructor(field: Searchable) {
    this.field = field
  }

  /**
   * Set the field-level auth directive.
   *
   * @param rules - A closure to build the authentication rules.
   */
  public auth(rules: AuthRuleF): AuthDefinition {
    return new AuthDefinition(this, rules)
  }

  /**
   * Set the field-level cache directive.
   *
   * @param params - The cache definition parameters.
   */
  public cache(params: FieldCacheParams): CacheDefinition {
    return new CacheDefinition(this, new FieldLevelCache(params))
  }

  /**
   * Make the field unique.
   *
   * @param scope - Additional fields to be added to the constraint.
   */
  public unique(scope?: string[]): UniqueDefinition {
    return new UniqueDefinition(this, scope)
  }

  public toString(): string {
    return `${this.field} @search`
  }
}
