import { AuthRuleF } from '../auth'
import { Enum } from '../enum'
import { FieldType } from '../field/typedefs'
import { AuthDefinition } from './auth'
import { LengthLimitedStringDefinition } from './length-limited-string'
import { ResolverDefinition } from './resolver'
import { ScalarDefinition, DefaultValueType } from './scalar'
import { UniqueDefinition } from './unique'

export class DefaultDefinition {
  defaultValue: DefaultValueType
  scalar: ScalarDefinition | LengthLimitedStringDefinition

  constructor(
    scalar: ScalarDefinition | LengthLimitedStringDefinition,
    defaultValue: DefaultValueType
  ) {
    this.defaultValue = defaultValue
    this.scalar = scalar
  }

  public unique(): UniqueDefinition {
    return new UniqueDefinition(this)
  }

  public optional(): DefaultDefinition {
    this.scalar.optional()
    return this
  }

  public auth(rules: AuthRuleF): AuthDefinition {
    return new AuthDefinition(this, rules)
  }

  public resolver(name: string): ResolverDefinition {
    return new ResolverDefinition(this, name)
  }

  public toString(): string {
    return `${this.scalar} @default(value: ${renderDefault(
      this.defaultValue,
      this.scalar.fieldTypeVal()
    )})`
  }
}

export function renderDefault(val: any, fieldType: FieldType | Enum) {
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
