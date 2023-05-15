import { RequireAtLeastOne } from 'type-fest'
import { Enum } from '../enum'
import {
  BooleanListDefinition,
  DateListDefinition,
  ListDefinition,
  NumberListDefinition,
  StringListDefinition
} from './list'
import { ScalarType, Searchable } from '..'

interface FieldLength {
  min?: number
  max?: number
}

export enum FieldType {
  String = 'String',
  Int = 'Int',
  Email = 'Email',
  ID = 'ID',
  Float = 'Float',
  Boolean = 'Boolean',
  Date = 'Date',
  DateTime = 'DateTime',
  IPAddress = 'IPAddress',
  Timestamp = 'Timestamp',
  URL = 'URL',
  JSON = 'JSON',
  PhoneNumber = 'PhoneNumber'
}

export class SearchDefinition {
  field: Searchable

  constructor(field: Searchable) {
    this.field = field
  }

  public toString(): string {
    return `${this.field} @search`
  }
}

type UniqueScalarType =
  | ScalarDefinition
  | DefaultDefinition
  | SearchDefinition
  | LengthLimitedStringDefinition

export class UniqueDefinition {
  compoundScope?: string[]
  scalar: UniqueScalarType

  constructor(scalar: UniqueScalarType, scope?: string[]) {
    this.scalar = scalar
    this.compoundScope = scope
  }

  public search(): SearchDefinition {
    return new SearchDefinition(this)
  }

  public toString(): string {
    const scope = this.compoundScope?.map((field) => `"${field}"`).join(', ')
    const scopeArray = scope ? `[${scope}]` : null

    return scopeArray
      ? `${this.scalar} @unique(fields: ${scopeArray})`
      : `${this.scalar} @unique`
  }
}

export class DefaultDefinition {
  defaultValue: ScalarType
  scalar: ScalarDefinition | LengthLimitedStringDefinition

  constructor(
    scalar: ScalarDefinition | LengthLimitedStringDefinition,
    defaultValue: ScalarType
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

  public toString(): string {
    return `${this.scalar} @default(value: ${renderDefault(
      this.defaultValue,
      this.scalar.fieldTypeVal()
    )})`
  }
}

export class ScalarDefinition {
  fieldType: FieldType | Enum
  isOptional: boolean
  defaultValue?: ScalarType

  constructor(fieldType: FieldType | Enum) {
    this.fieldType = fieldType
    this.isOptional = false
  }

  public optional(): ScalarDefinition {
    this.isOptional = true

    return this
  }

  public unique(scope?: string[]): UniqueDefinition {
    return new UniqueDefinition(this, scope)
  }

  public search(): SearchDefinition {
    return new SearchDefinition(this)
  }

  public list(): ListDefinition {
    return new ListDefinition(this)
  }

  fieldTypeVal(): FieldType | Enum {
    return this.fieldType
  }

  public toString(): string {
    const required = this.isOptional ? '' : '!'

    let fieldType
    if (this.fieldType instanceof Enum) {
      fieldType = this.fieldType.name
    } else {
      fieldType = this.fieldType.toString()
    }

    return `${fieldType}${required}`
  }
}

export class StringDefinition extends ScalarDefinition {
  public default(val: string): DefaultDefinition {
    return new DefaultDefinition(this, val)
  }

  public length(
    fieldLength: RequireAtLeastOne<FieldLength, 'min' | 'max'>
  ): LengthLimitedStringDefinition {
    return new LengthLimitedStringDefinition(this, fieldLength)
  }

  public list(): StringListDefinition {
    return new StringListDefinition(this)
  }
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

  fieldTypeVal(): FieldType | Enum {
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

export class NumberDefinition extends ScalarDefinition {
  public default(val: number): DefaultDefinition {
    return new DefaultDefinition(this, val)
  }

  public list(): NumberListDefinition {
    return new NumberListDefinition(this)
  }
}

export class BooleanDefinition extends ScalarDefinition {
  public default(val: boolean): DefaultDefinition {
    return new DefaultDefinition(this, val)
  }

  public list(): BooleanListDefinition {
    return new BooleanListDefinition(this)
  }
}

export class DateDefinition extends ScalarDefinition {
  public default(val: Date): DefaultDefinition {
    return new DefaultDefinition(this, val)
  }

  public list(): DateListDefinition {
    return new DateListDefinition(this)
  }
}

export class ObjectDefinition extends ScalarDefinition {
  public default(val: object): DefaultDefinition {
    return new DefaultDefinition(this, val)
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
