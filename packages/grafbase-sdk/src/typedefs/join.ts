import { AuthRuleF } from '../auth'
import { AuthDefinition } from './auth'
import { CacheDefinition, FieldCacheParams, FieldLevelCache } from './cache'
import { DefaultDefinition } from './default'
import { ReferenceDefinition } from './reference'
import { ScalarDefinition } from './scalar'
import { EnumDefinition } from './enum'
import { TagDefinition } from './tag'
import { InaccessibleDefinition } from './inaccessible'
import { ShareableDefinition } from './shareable'
import { OverrideDefinition } from './override'
import { ProvidesDefinition } from './provides'
import { DeprecatedDefinition } from './deprecated'
import { InputType } from '../query'

/**
 * A list of field types that can hold a `@join` attribute.
 */
export type Joinable =
  | ScalarDefinition
  | DefaultDefinition
  | ReferenceDefinition
  | CacheDefinition
  | EnumDefinition<any, any>
  | TagDefinition
  | InaccessibleDefinition
  | ShareableDefinition
  | OverrideDefinition
  | ProvidesDefinition
  | DeprecatedDefinition

export class JoinDefinition {
  private field: Joinable
  private select: string
  private _arguments: Record<string, InputType>

  constructor(field: Joinable, select: string) {
    this.field = field
    this.select = select
    this._arguments = {}
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
   * Add arguments to this field that will be available in the join string
   *
   * @param args - The arguments for this field
   */
  public arguments(args: Record<string, InputType>): JoinDefinition {
    this._arguments = args
    return this
  }

  public get allArguments(): Record<string, InputType> {
    return { ...this._arguments, ...this.field.allArguments }
  }

  public toString(): string {
    return `${this.field} @join(select: "${this.select}")`
  }
}
