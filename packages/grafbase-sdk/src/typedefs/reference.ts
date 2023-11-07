import { AuthRuleF } from '../auth'
import { ListDefinition } from './list'
import { Type } from '../type'
import { AuthDefinition } from './auth'
import { ResolverDefinition } from './resolver'
import { MapDefinition } from './map'
import { JoinDefinition } from './join'

export class ReferenceDefinition {
  private referencedType: string
  private isOptional: boolean

  constructor(referencedType: Type | string) {
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

  public toString(): string {
    const required = this.isOptional ? '' : '!'

    return `${this.referencedType}${required}`
  }
}
