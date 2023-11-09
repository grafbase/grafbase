import { AuthRuleF } from '../auth'
import { Enum, EnumShape } from '../enum'
import { AuthDefinition } from './auth'
import { CacheDefinition, FieldCacheParams, FieldLevelCache } from './cache'
import { DefaultDefinition } from './default'
import { DeprecatedDefinition } from './deprecated'
import { InaccessibleDefinition } from './inaccessible'
import { JoinDefinition } from './join'
import { ListDefinition } from './list'
import { MapDefinition } from './map'
import { OverrideDefinition } from './override'
import { ProvidesDefinition } from './provides'
import { ResolverDefinition } from './resolver'
import { SearchDefinition } from './search'
import { ShareableDefinition } from './shareable'
import { UniqueDefinition } from './unique'

export class EnumDefinition<T extends string, U extends EnumShape<T>> {
  private enumName: string
  private enumVariants: U
  private isOptional: boolean

  constructor(referencedEnum: Enum<T, U>) {
    this.enumName = referencedEnum.name
    this.enumVariants = referencedEnum.variants
    this.isOptional = false
  }

  /**
   * Set the field optional.
   */
  public optional(): this {
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
   * Set the default value of the field.
   *
   * @param value - The value written to the database.
   */
  public default(value: U[number]): DefaultDefinition {
    return new DefaultDefinition(this, value)
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

  public toString(): string {
    const required = this.isOptional ? '' : '!'

    return `${this.enumName}${required}`
  }

  fieldTypeVal(): Enum<T, U> {
    return new Enum(this.enumName, this.enumVariants)
  }
}
