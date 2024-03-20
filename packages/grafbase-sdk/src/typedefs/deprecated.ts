import { AuthRuleF } from '../auth'
import { AuthDefinition } from './auth'
import { CacheDefinition, FieldCacheParams, FieldLevelCache } from './cache'
import { DefaultDefinition } from './default'
import { ReferenceDefinition } from './reference'
import { ScalarDefinition } from './scalar'
import { EnumDefinition } from './enum'
import { escapeString } from '../utils'
import { ResolverDefinition } from './resolver'
import { JoinDefinition } from './join'
import { InputType } from '../query'

/**
 * A list of field types that can hold a `@deprecated` attribute.
 */
export type Deprecatable =
  | ScalarDefinition
  | DefaultDefinition
  | ReferenceDefinition
  | EnumDefinition<any, any>

export class DeprecatedDefinition {
  private field: Deprecatable
  private reason: string | null

  constructor(field: Deprecatable, reason: string | null) {
    this.field = field
    this.reason = reason
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
   * Attach a resolver function to the field.
   *
   * @param name - The name of the resolver function file without the extension or directory.
   */
  public resolver(name: string): ResolverDefinition {
    return new ResolverDefinition(this, name)
  }

  /**
   * Attach a join function to the field.
   *
   * @param select - The field selection string to join onto this field
   */
  public join(select: string): JoinDefinition {
    return new JoinDefinition(this, select)
  }

  /**
   * Set the field-level cache directive.
   *
   * @param params - The cache definition parameters.
   */
  public cache(params: FieldCacheParams): CacheDefinition {
    return new CacheDefinition(this, new FieldLevelCache(params))
  }

  public get allArguments(): Record<string, InputType> {
    return { ...this.field.allArguments }
  }

  public toString(): string {
    const reason = this.reason ? `(reason: "${escapeString(this.reason)}")` : ''

    return `${this.field} @deprecated${reason}`
  }
}
