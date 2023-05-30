import { AuthRuleF, AuthRules } from '../auth'
import { ReferenceDefinition } from './reference'
import { RelationDefinition } from '../relation'
import { CacheDefinition, FieldCacheParams, FieldLevelCache } from './cache'
import { DefaultDefinition } from './default'
import { LengthLimitedStringDefinition } from './length-limited-string'
import { ResolverDefinition } from './resolver'
import { ScalarDefinition } from './scalar'
import { SearchDefinition } from './search'
import { UniqueDefinition } from './unique'
import { EnumDefinition } from './enum'

export type Authenticable =
  | ScalarDefinition
  | UniqueDefinition
  | DefaultDefinition
  | SearchDefinition
  | ReferenceDefinition
  | LengthLimitedStringDefinition
  | RelationDefinition
  | CacheDefinition
  | ResolverDefinition
  | EnumDefinition<any, any>

export class AuthDefinition {
  field: Authenticable
  authRules: AuthRules

  constructor(field: Authenticable, rules: AuthRuleF) {
    const authRules = new AuthRules()
    rules(authRules)

    this.authRules = authRules
    this.field = field
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
    // In field definition, concatenate all rules into one row.
    const rules = this.authRules.toString().replace(/\s\s+/g, ' ')

    return `${this.field} @auth(rules: ${rules})`
  }
}
