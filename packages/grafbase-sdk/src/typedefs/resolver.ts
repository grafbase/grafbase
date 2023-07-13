import { AuthRuleF } from '../auth'
import { AuthDefinition } from './auth'
import { CacheDefinition, FieldCacheParams, FieldLevelCache } from './cache'
import { DefaultDefinition } from './default'
import { ReferenceDefinition } from './reference'
import { ScalarDefinition } from './scalar'
import { EnumDefinition } from './enum'
import { MapDefinition } from './map'

/**
 * A list of field types that can hold a `@resolver` attribute.
 */
export type Resolvable =
  | ScalarDefinition
  | DefaultDefinition
  | ReferenceDefinition
  | CacheDefinition
  | EnumDefinition<any, any>

export class ResolverDefinition {
  private field: Resolvable
  private resolver: string

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
   * Set the field-level cache directive.
   *
   * @param params - The cache definition parameters.
   */
  public cache(params: FieldCacheParams): CacheDefinition {
    return new CacheDefinition(this, new FieldLevelCache(params))
  }

  /**
   * Sets the name of the field in the database, if different than the name of the field.
   *
   * @param name - The mapped name
   */
  public map(name: string): MapDefinition {
    return new MapDefinition(this, name)
  }

  public toString(): string {
    return `${this.field} @resolver(name: "${this.resolver}")`
  }
}
