import { AuthRuleF } from '../auth'
import { ListDefinition } from './list'
import { Type } from '../type'
import { AuthDefinition } from './auth'
import { ResolverDefinition } from './resolver'
import { MapDefinition } from './map'
import { JoinDefinition } from './join'
import { DeprecatedDefinition } from './deprecated'
import { InaccessibleDefinition } from './inaccessible'
import { ShareableDefinition } from './shareable'
import { OverrideDefinition } from './override'
import { ProvidesDefinition } from './provides'
import { InputType } from '../query'
import { Union } from '../union'

export class ReferenceDefinition {
  private referencedType: string
  private isOptional: boolean

  constructor(referencedType: Type | Union | string) {
    this.referencedType =
      typeof referencedType === 'string' ? referencedType : referencedType.name
    this.isOptional = false
  }

  /**
   * Set the field optional.
   */
  public optional(): ReferenceDefinition {
    this.isOptional = true

    return this
  }

  /**
   * Allow multiple scalars to be used as values for the field.
   */
  public list(): ListDefinition {
    return new ListDefinition(this)
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
   * Set the field-level deprecated directive.
   *
   * @param rules - A closure to build the authentication rules.
   */
  public deprecated(reason?: string): DeprecatedDefinition {
    return new DeprecatedDefinition(this, reason ?? null)
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
   * Sets the name of the field in the database, if different than the name of the field.
   */
  public mapped(name: string): MapDefinition {
    return new MapDefinition(this, name)
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

  public get allArguments(): Record<string, InputType> {
    return {}
  }

  public toString(): string {
    const required = this.isOptional ? '' : '!'

    return `${this.referencedType}${required}`
  }
}
