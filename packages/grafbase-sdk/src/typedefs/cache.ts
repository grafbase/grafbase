import { AuthRuleF } from '../auth'
import {
  AccessScope,
  MutationInvalidation,
  renderMutationInvalidation,
  renderAccessScope
} from '../cache'
import { AuthDefinition } from './auth'
import { DefaultDefinition } from './default'
import { DeprecatedDefinition } from './deprecated'
import { EnumDefinition } from './enum'
import { InaccessibleDefinition } from './inaccessible'
import { JoinDefinition } from './join'
import { LengthLimitedStringDefinition } from './length-limited-string'
import { MapDefinition } from './map'
import { OverrideDefinition } from './override'
import { ProvidesDefinition } from './provides'
import { ResolverDefinition } from './resolver'
import { ScalarDefinition } from './scalar'
import { ShareableDefinition } from './shareable'
import { TagDefinition } from './tag'
import { UniqueDefinition } from './unique'

export type Cacheable =
  | ScalarDefinition
  | AuthDefinition
  | DefaultDefinition
  | ResolverDefinition
  | LengthLimitedStringDefinition
  | UniqueDefinition
  | EnumDefinition<any, any>
  | JoinDefinition
  | TagDefinition
  | InaccessibleDefinition
  | ShareableDefinition
  | OverrideDefinition
  | ProvidesDefinition
  | DeprecatedDefinition

export interface TypeCacheParams {
  maxAge: number
  staleWhileRevalidate?: number
  mutationInvalidation?: MutationInvalidation
  scopes?: AccessScope[]
}

export interface FieldCacheParams {
  maxAge: number
  staleWhileRevalidate?: number
  scopes?: AccessScope[]
}

export class TypeLevelCache {
  private params: TypeCacheParams

  constructor(params: TypeCacheParams) {
    this.params = params
  }

  public toString(): string {
    const maxAge = `maxAge: ${this.params.maxAge}`

    const staleWhileRevalidate = this.params.staleWhileRevalidate
      ? `, staleWhileRevalidate: ${this.params.staleWhileRevalidate}`
      : ''

    const mutationInvalidation = this.params.mutationInvalidation
      ? `, mutationInvalidation: ${renderMutationInvalidation(
          this.params.mutationInvalidation
        )}`
      : ''

    const scopes = this.params.scopes
      ? `, scopes: [${this.params.scopes
          .map((scope) => renderAccessScope(scope))
          .join(', ')}]`
      : ''

    return `@cache(${maxAge}${staleWhileRevalidate}${mutationInvalidation}${scopes})`
  }
}

export class FieldLevelCache {
  private params: FieldCacheParams

  constructor(params: FieldCacheParams) {
    this.params = params
  }

  public toString(): string {
    const maxAge = `maxAge: ${this.params.maxAge}`

    const staleWhileRevalidate = this.params.staleWhileRevalidate
      ? `, staleWhileRevalidate: ${this.params.staleWhileRevalidate}`
      : ''

    const scopes = this.params.scopes
      ? `, scopes: [${this.params.scopes
          .map((scope) => renderAccessScope(scope))
          .join(', ')}]`
      : ''

    return `@cache(${maxAge}${staleWhileRevalidate}${scopes})`
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
   * Sets the name of the field in the database, if different than the name of the field.
   *
   * @param name - The mapped name
   */
  public mapped(name: string): MapDefinition {
    return new MapDefinition(this, name)
  }

  public toString(): string {
    return `${this.field} ${this.attribute}`
  }
}
