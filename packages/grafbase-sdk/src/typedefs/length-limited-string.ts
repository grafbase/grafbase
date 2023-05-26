import { RequireAtLeastOne } from 'type-fest'
import { FieldType } from '../typedefs'
import { UniqueDefinition } from './unique'
import { DefaultDefinition } from './default'
import { Enum } from '../enum'
import { StringDefinition } from './scalar'
import { SearchDefinition } from './search'
import { AuthRuleF } from '../auth'
import { AuthDefinition } from './auth'
import { CacheDefinition, FieldCacheParams, FieldLevelCache } from './cache'

export interface FieldLength {
  min?: number
  max?: number
}

export class LengthLimitedStringDefinition {
  fieldLength: RequireAtLeastOne<FieldLength, 'min' | 'max'>
  scalar: StringDefinition

  constructor(
    scalar: StringDefinition,
    fieldLength: RequireAtLeastOne<FieldLength, 'min' | 'max'>
  ) {
    this.fieldLength = fieldLength
    this.scalar = scalar
  }

  public unique(scope?: string[]): UniqueDefinition {
    return new UniqueDefinition(this, scope)
  }

  public search(): SearchDefinition {
    return new SearchDefinition(this)
  }

  public default(val: string): DefaultDefinition {
    return new DefaultDefinition(this, val)
  }

  public optional(): LengthLimitedStringDefinition {
    this.scalar.optional()

    return this
  }

  public auth(rules: AuthRuleF): AuthDefinition {
    return new AuthDefinition(this, rules)
  }

  public cache(params: FieldCacheParams): CacheDefinition {
    return new CacheDefinition(this, new FieldLevelCache(params))
  }

  fieldTypeVal(): FieldType | Enum<any, any> {
    return this.scalar.fieldType
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
}
