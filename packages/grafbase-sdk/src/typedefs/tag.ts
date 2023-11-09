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
import { InaccessibleDefinition } from './inaccessible'
import { ShareableDefinition } from './shareable'
import { OverrideDefinition } from './override'
import { ProvidesDefinition } from './provides'
import { DeprecatedDefinition } from './deprecated'

/**
 * A list of field types that can hold a `@tag` attribute.
 */
export type Taggable =
  | ScalarDefinition
  | DefaultDefinition
  | ReferenceDefinition
  | EnumDefinition<any, any>
  | TagDefinition
  | InaccessibleDefinition
  | ShareableDefinition
  | OverrideDefinition
  | ProvidesDefinition
  | DeprecatedDefinition

export class TagDefinition {
  private field: Taggable
  private name: string

  constructor(field: Taggable, name: string) {
    this.field = field
    this.name = name
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
   * Adds a tag to this field
   *
   * @param tag - The tag to add
   */
  public tag(tag: string): TagDefinition {
    return new TagDefinition(this, tag)
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
   * Set the field-level inaccessible directive.
   */
  public inaccessible(): InaccessibleDefinition {
    return new InaccessibleDefinition(this)
  }

  /**
   * Set the field-level shareable directive.
   */
  public shareable(): ShareableDefinition {
    return new ShareableDefinition(this)
  }

  /**
   * Set the field-level override directive.
   */
  public override(from: string): OverrideDefinition {
    return new OverrideDefinition(this, from)
  }

  /**
   * Set the field-level provides directive.
   */
  public provides(fields: string): ProvidesDefinition {
    return new ProvidesDefinition(this, fields)
  }

  public toString(): string {
    return `${this.field} @tag(name: "${escapeString(this.name)}")`
  }
}
