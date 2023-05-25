import { AuthRuleF } from '../auth'
import { AuthDefinition } from './auth'
import { CacheDefinition, CacheParams, TypeLevelCache } from './cache'
import { SearchDefinition } from './search'
import { DefaultDefinition } from './default'
import { ReferenceDefinition } from './reference'
import { ScalarDefinition } from './scalar'
import { UniqueDefinition } from './unique'

/**
 * A list of field types that can hold a `@resolver` attribute.
 */
export type Resolvable =
  | ScalarDefinition
  | UniqueDefinition
  | DefaultDefinition
  | ReferenceDefinition
  | CacheDefinition

export class ResolverDefinition {
  field: Resolvable
  resolver: string

  constructor(field: Resolvable, resolver: string) {
    this.field = field
    this.resolver = resolver
  }

  public auth(rules: AuthRuleF): AuthDefinition {
    return new AuthDefinition(this, rules)
  }

  public search(): SearchDefinition {
    return new SearchDefinition(this)
  }

  public unique(scope?: string[]): UniqueDefinition {
    return new UniqueDefinition(this, scope)
  }

  public cache(params: CacheParams): CacheDefinition {
    return new CacheDefinition(this, new TypeLevelCache(params))
  }

  public toString(): string {
    return `${this.field} @resolver(name: "${this.resolver}")`
  }
}
