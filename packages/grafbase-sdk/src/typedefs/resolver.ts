import { AuthRuleF } from '../auth'
import { AuthDefinition } from './auth'
import { CacheDefinition, FieldCacheParams, FieldLevelCache } from './cache'
import { SearchDefinition } from './search'
import { DefaultDefinition } from './default'
import { ReferenceDefinition } from './reference'
import { ScalarDefinition } from './scalar'
import { UniqueDefinition } from './unique'
import { EnumDefinition } from './enum'

/**
 * A list of field types that can hold a `@resolver` attribute.
 */
export type Resolvable =
  | ScalarDefinition
  | UniqueDefinition
  | DefaultDefinition
  | ReferenceDefinition
  | CacheDefinition
  | EnumDefinition<any, any>

export class ResolverDefinition {
  field: Resolvable
  resolver: string

  constructor(field: Resolvable, resolver: string) {
    this.field = field
    this.resolver = resolver
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
   * Make the field searchable.
   */
  public search(): SearchDefinition {
    return new SearchDefinition(this)
  }

  /**
   * Make the field unique.
   *
   * @param scope - Additional fields to be added to the constraint.
   */
  public unique(scope?: string[]): UniqueDefinition {
    return new UniqueDefinition(this, scope)
  }

  /**
   * Set the field-level cache directive.
   *
   * @param params - The cache definition parameters.
   */
  public cache(params: FieldCacheParams): CacheDefinition {
    return new CacheDefinition(this, new FieldLevelCache(params))
  }

  public toString(): string {
    return `${this.field} @resolver(name: "${this.resolver}")`
  }
}
