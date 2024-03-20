import { RequireAtLeastOne } from 'type-fest'
import { FieldType } from '../typedefs'
import { UniqueDefinition } from './unique'
import { DefaultDefinition } from './default'
import { Enum } from '../enum'
import { StringDefinition } from './scalar'
import { AuthRuleF } from '../auth'
import { AuthDefinition } from './auth'
import { CacheDefinition, FieldCacheParams, FieldLevelCache } from './cache'
import { StringListDefinition } from './list'
import { MapDefinition } from './map'
import { InputType } from '../query'

export interface FieldLength {
  min?: number
  max?: number
}

export type LengthLimitedField = StringDefinition | StringListDefinition

export class LengthLimitedStringDefinition {
  private fieldLength: RequireAtLeastOne<FieldLength, 'min' | 'max'>
  private scalar: LengthLimitedField

  constructor(
    scalar: LengthLimitedField,
    fieldLength: RequireAtLeastOne<FieldLength, 'min' | 'max'>
  ) {
    this.fieldLength = fieldLength
    this.scalar = scalar
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
  public default(val: string): DefaultDefinition {
    return new DefaultDefinition(this, val)
  }

  /**
   * Set the field optional.
   */
  public optional(): LengthLimitedStringDefinition {
    this.scalar.optional()

    return this
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
   */
  public mapped(name: string): MapDefinition {
    return new MapDefinition(this, name)
  }

  public toString(): string {
    const length = this.fieldLength

    if (length.min != null && length.max != null) {
      return `${this.scalar} @length(min: ${length.min}, max: ${length.max})`
    } else if (length.min != null) {
      return `${this.scalar} @length(min: ${length.min})`
    } else {
      return `${this.scalar} @length(max: ${length.max})`
    }
  }

  fieldTypeVal(): FieldType | Enum<any, any> {
    return this.scalar.fieldType
  }

  public get allArguments(): Record<string, InputType> {
    return this.scalar.allArguments
  }
}
