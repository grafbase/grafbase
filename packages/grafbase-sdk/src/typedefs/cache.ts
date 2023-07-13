import { AuthRuleF } from '../auth'
import { MutationInvalidation, renderMutationInvalidation } from '../cache'
import { AuthDefinition } from './auth'
import { DefaultDefinition } from './default'
import { EnumDefinition } from './enum'
import { LengthLimitedStringDefinition } from './length-limited-string'
import { MapDefinition } from './map'
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
  | EnumDefinition<any, any>

export interface TypeCacheParams {
  maxAge: number
  staleWhileRevalidate?: number
  mutationInvalidation?: MutationInvalidation
}

export interface FieldCacheParams {
  maxAge: number
  staleWhileRevalidate?: number
}

export class TypeLevelCache {
  private params: TypeCacheParams

  constructor(params: TypeCacheParams) {
    this.params = params
  }

  public toString(): string {
    let maxAge = `maxAge: ${this.params.maxAge}`

    let staleWhileRevalidate = this.params.staleWhileRevalidate
      ? `, staleWhileRevalidate: ${this.params.staleWhileRevalidate}`
      : ''

    let mutationInvalidation = this.params.mutationInvalidation
      ? `, mutationInvalidation: ${renderMutationInvalidation(
          this.params.mutationInvalidation
        )}`
      : ''

    return `@cache(${maxAge}${staleWhileRevalidate}${mutationInvalidation})`
  }
}

export class FieldLevelCache {
  private params: FieldCacheParams

  constructor(params: FieldCacheParams) {
    this.params = params
  }

  public toString(): string {
    let maxAge = `maxAge: ${this.params.maxAge}`

    let staleWhileRevalidate = this.params.staleWhileRevalidate
      ? `, staleWhileRevalidate: ${this.params.staleWhileRevalidate}`
      : ''

    return `@cache(${maxAge}${staleWhileRevalidate})`
  }
}

export class CacheDefinition {
  private attribute: FieldLevelCache
  private field: Cacheable

  constructor(field: Cacheable, attribute: FieldLevelCache) {
    this.attribute = attribute
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
   * Make the field searchable.
   */
  public search(): SearchDefinition {
    return new SearchDefinition(this)
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
    return `${this.field} ${this.attribute}`
  }
}
