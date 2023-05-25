import { AuthRuleF } from '../auth'
import { ListDefinition } from '../field/list'
import { AuthDefinition } from './auth'
import { CacheDefinition, CacheParams, TypeLevelCache } from './cache'
import { LengthLimitedStringDefinition } from './length-limited-string'
import { ResolverDefinition } from './resolver'
import { ScalarDefinition } from './scalar'
import { UniqueDefinition } from './unique'

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
  | ResolverDefinition

export class SearchDefinition {
  field: Searchable

  constructor(field: Searchable) {
    this.field = field
  }

  public auth(rules: AuthRuleF): AuthDefinition {
    return new AuthDefinition(this, rules)
  }

  public cache(params: CacheParams): CacheDefinition {
    return new CacheDefinition(this, new TypeLevelCache(params))
  }

  public unique(scope?: string[]): UniqueDefinition {
    return new UniqueDefinition(this, scope)
  }

  public toString(): string {
    return `${this.field} @search`
  }
}
