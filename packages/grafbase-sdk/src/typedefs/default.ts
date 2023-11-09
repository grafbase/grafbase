import { AuthRuleF } from '../auth'
import { Enum } from '../enum'
import { FieldType } from '../typedefs'
import { AuthDefinition } from './auth'
import { CacheDefinition, FieldCacheParams, FieldLevelCache } from './cache'
import { LengthLimitedStringDefinition } from './length-limited-string'
import { ScalarDefinition } from './scalar'
import { UniqueDefinition } from './unique'
import { EnumDefinition } from './enum'
import { MapDefinition } from './map'
import { InaccessibleDefinition } from './inaccessible'
import { ShareableDefinition } from './shareable'
import { OverrideDefinition } from './override'
import { ProvidesDefinition } from './provides'

export type DefaultValueType = string | number | Date | object | boolean

export type DefaultFieldShape =
  | ScalarDefinition
  | LengthLimitedStringDefinition
  | EnumDefinition<any, any>

export class DefaultDefinition {
  protected _defaultValue: DefaultValueType
  protected _scalar: DefaultFieldShape

  constructor(scalar: DefaultFieldShape, defaultValue: DefaultValueType) {
    this._defaultValue = defaultValue
    this._scalar = scalar
  }

  /**
   * The default value.
   */
  public get defaultValue(): DefaultValueType {
    return this._defaultValue
  }

  /**
   * The default type of the default value.
   */
  public get scalar(): DefaultFieldShape {
    return this._scalar
  }

  /**
   * Make the field unique.
   */
  public unique(): UniqueDefinition {
    return new UniqueDefinition(this)
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
    return `${this._scalar} @default(value: ${renderDefault(
      this._defaultValue,
      this._scalar.fieldTypeVal()
    )})`
  }
}

export function renderDefault(val: any, fieldType: FieldType | Enum<any, any>) {
  const pad2 = (n: number): string => {
    return n < 10 ? `0${n}` : `${n}`
  }

  const pad4 = (n: number): string => {
    if (n < 10) {
      return `000${n}`
    } else if (n < 100) {
      return `00${n}`
    } else if (n < 1000) {
      return `0${n}`
    } else {
      return `${n}`
    }
  }

  if (fieldType instanceof Enum) {
    return val.toString()
  } else {
    switch (fieldType) {
      case FieldType.String:
      case FieldType.ID:
      case FieldType.Email:
      case FieldType.PhoneNumber:
      case FieldType.IPAddress:
      case FieldType.URL: {
        return `"${val}"`
      }
      case FieldType.Date: {
        const year = pad4(val.getUTCFullYear())
        const month = pad2(val.getUTCMonth() + 1)
        const date = pad2(val.getUTCDate())

        return `"${year}-${month}-${date}"`
      }
      case FieldType.DateTime:
        const year = pad4(val.getUTCFullYear())
        const month = pad2(val.getUTCMonth() + 1)
        const date = pad2(val.getUTCDate())
        const hours = pad2(val.getUTCHours())
        const minutes = pad2(val.getUTCMinutes())
        const seconds = pad2(val.getUTCSeconds())

        return `"${year}-${month}-${date}T${hours}:${minutes}:${seconds}Z"`
      default: {
        return val.toString()
      }
    }
  }
}
