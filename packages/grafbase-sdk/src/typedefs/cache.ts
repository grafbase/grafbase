import { AuthRuleF } from '../auth'
import { AuthDefinition } from './auth'
import { DefaultDefinition } from './default'
import { LengthLimitedStringDefinition } from './length-limited-string'
import { ResolverDefinition } from './resolver'
import { ScalarDefinition } from './scalar'
import { SearchDefinition } from './search'
import { UniqueDefinition } from './unique'

export type Cacheable =
  | ScalarDefinition
  | AuthDefinition
  | DefaultDefinition
  | ResolverDefinition
  | LengthLimitedStringDefinition
  | SearchDefinition
  | UniqueDefinition

export interface CacheParams {
  maxAge: number
  staleWhileRevalidate: number
}

export class TypeLevelCache {
  params: CacheParams

  constructor(params: CacheParams) {
    this.params = params
  }

  public toString(): string {
    return `@cache(maxAge: ${this.params.maxAge}, staleWhileRevalidate: ${this.params.staleWhileRevalidate})`
  }
}

export class CacheDefinition {
  attribute: TypeLevelCache
  field: Cacheable

  constructor(field: Cacheable, attribute: TypeLevelCache) {
    this.attribute = attribute
    this.field = field
  }

  public auth(rules: AuthRuleF): AuthDefinition {
    return new AuthDefinition(this, rules)
  }

  public search(): SearchDefinition {
    return new SearchDefinition(this)
  }

  public toString(): string {
    return `${this.field} ${this.attribute}`
  }
}
