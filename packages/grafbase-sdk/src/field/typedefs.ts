import { RequireAtLeastOne } from 'type-fest'
import { Enum } from '../enum'
import {
  BooleanListDef,
  DateListDef,
  ListDef,
  NumberListDef,
  StringListDef
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

export class SearchDef {
  field: Searchable

  constructor(field: Searchable) {
    this.field = field
  }

  public toString(): string {
    return `${this.field} @search`
  }
}

type UniqueScalarType =
  | ScalarDef
  | DefaultDef
  | SearchDef
  | LengthLimitedStringDef

export class UniqueDef {
  compoundScope?: string[]
  scalar: UniqueScalarType

  constructor(scalar: UniqueScalarType, scope?: string[]) {
    this.scalar = scalar
    this.compoundScope = scope
  }

  public search(): SearchDef {
    return new SearchDef(this)
  }

  public toString(): string {
    const scope = this.compoundScope?.map((field) => `"${field}"`).join(', ')
    const scopeArray = scope ? `[${scope}]` : null

    return scopeArray
      ? `${this.scalar} @unique(fields: ${scopeArray})`
      : `${this.scalar} @unique`
  }
}

export class DefaultDef {
  defaultValue: ScalarType
  scalar: ScalarDef | LengthLimitedStringDef

  constructor(
    scalar: ScalarDef | LengthLimitedStringDef,
    defaultValue: ScalarType
  ) {
    this.defaultValue = defaultValue
    this.scalar = scalar
  }

  public unique(): UniqueDef {
    return new UniqueDef(this)
  }

  public optional(): DefaultDef {
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

export class ScalarDef {
  fieldType: FieldType | Enum
  isOptional: boolean
  defaultValue?: ScalarType

  constructor(fieldType: FieldType | Enum) {
    this.fieldType = fieldType
    this.isOptional = false
  }

  public optional(): ScalarDef {
    this.isOptional = true

    return this
  }

  public unique(scope?: string[]): UniqueDef {
    return new UniqueDef(this, scope)
  }

  public search(): SearchDef {
    return new SearchDef(this)
  }

  public list(): ListDef {
    return new ListDef(this)
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

export class StringDef extends ScalarDef {
  public default(val: string): DefaultDef {
    return new DefaultDef(this, val)
  }

  public length(
    fieldLength: RequireAtLeastOne<FieldLength, 'min' | 'max'>
  ): LengthLimitedStringDef {
    return new LengthLimitedStringDef(this, fieldLength)
  }

  public list(): StringListDef {
    return new StringListDef(this)
  }
}

export class LengthLimitedStringDef {
  fieldLength: RequireAtLeastOne<FieldLength, 'min' | 'max'>
  scalar: StringDef

  constructor(
    scalar: StringDef,
    fieldLength: RequireAtLeastOne<FieldLength, 'min' | 'max'>
  ) {
    this.fieldLength = fieldLength
    this.scalar = scalar
  }

  public unique(scope?: string[]): UniqueDef {
    return new UniqueDef(this, scope)
  }

  public search(): SearchDef {
    return new SearchDef(this)
  }

  public default(val: string): DefaultDef {
    return new DefaultDef(this, val)
  }

  public optional(): LengthLimitedStringDef {
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

export class NumberDef extends ScalarDef {
  public default(val: number): DefaultDef {
    return new DefaultDef(this, val)
  }

  public list(): NumberListDef {
    return new NumberListDef(this)
  }
}

export class BooleanDef extends ScalarDef {
  public default(val: boolean): DefaultDef {
    return new DefaultDef(this, val)
  }

  public list(): BooleanListDef {
    return new BooleanListDef(this)
  }
}

export class DateDef extends ScalarDef {
  public default(val: Date): DefaultDef {
    return new DefaultDef(this, val)
  }

  public list(): DateListDef {
    return new DateListDef(this)
  }
}

export class ObjectDef extends ScalarDef {
  public default(val: object): DefaultDef {
    return new DefaultDef(this, val)
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
