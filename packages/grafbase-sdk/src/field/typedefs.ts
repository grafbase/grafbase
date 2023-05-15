import { RequireAtLeastOne } from "type-fest"
import { Enum } from "../enum"
import { GBooleanListDef, GDateListDef, GListDef, GNumberListDef, GStringListDef } from "./list"
import { ScalarType, Searchable } from ".."

interface FieldLength {
  min?: number
  max?: number
}

export enum FieldType {
  String = "String",
  Int = "Int",
  Email = "Email",
  ID = "ID",
  Float = "Float",
  Boolean = "Boolean",
  Date = "Date",
  DateTime = "DateTime",
  IPAddress = "IPAddress",
  Timestamp = "Timestamp",
  URL = "URL",
  JSON = "JSON",
  PhoneNumber = "PhoneNumber",
}

export class GSearchDef {
  field: Searchable

  constructor(field: Searchable) {
    this.field = field
  }

  public toString(): string {
    return `${this.field} @search`
  }
}

type UniqueScalarType = GScalarDef | GDefaultDef | GSearchDef | GLengthLimitedStringDef

export class GUniqueDef {
  compoundScope?: string[]
  scalar: UniqueScalarType

  constructor(scalar: UniqueScalarType, scope?: string[]) {
    this.scalar = scalar
    this.compoundScope = scope
  }

  public search(): GSearchDef {
    return new GSearchDef(this)
  }

  public toString(): string {
    const scope = this.compoundScope?.map((field) => `"${field}"`).join(", ")
    const scopeArray = scope ? `[${scope}]` : null

    return scopeArray ? `${this.scalar} @unique(fields: ${scopeArray})` : `${this.scalar} @unique`
  }
}

export class GDefaultDef {
  defaultValue: ScalarType
  scalar: GScalarDef | GLengthLimitedStringDef

  constructor(scalar: GScalarDef | GLengthLimitedStringDef, defaultValue: ScalarType) {
    this.defaultValue = defaultValue
    this.scalar = scalar
  }

  public unique(): GUniqueDef {
    return new GUniqueDef(this)
  }

  public optional(): GDefaultDef {
    this.scalar.optional()
    return this
  }

  public toString(): string {
    return `${this.scalar} @default(value: ${renderDefault(this.defaultValue, this.scalar.fieldTypeVal())})`
  }
}

export class GScalarDef {
  fieldType: FieldType | Enum
  isOptional: boolean
  defaultValue?: ScalarType

  constructor(fieldType: FieldType | Enum) {
    this.fieldType = fieldType
    this.isOptional = false
  }

  public optional(): GScalarDef {
    this.isOptional = true

    return this
  }

  public unique(scope?: string[]): GUniqueDef {
    return new GUniqueDef(this, scope)
  }

  public search(): GSearchDef {
    return new GSearchDef(this)
  }

  public list(): GListDef {
    return new GListDef(this)
  }

  fieldTypeVal(): FieldType | Enum {
    return this.fieldType
  }

  public toString(): string {
    const required = this.isOptional ? "" : "!"

    let fieldType
    if (this.fieldType instanceof Enum) {
      fieldType = this.fieldType.name
    } else {
      fieldType = this.fieldType.toString()
    }

    return `${fieldType}${required}`
  }
}

export class GStringDef extends GScalarDef {
  public default(val: string): GDefaultDef {
    return new GDefaultDef(this, val)
  }

  public length(fieldLength: RequireAtLeastOne<FieldLength, 'min' | 'max'>): GLengthLimitedStringDef {
    return new GLengthLimitedStringDef(this, fieldLength)
  }

  public list(): GStringListDef {
    return new GStringListDef(this)
  }
}

export class GLengthLimitedStringDef {
  fieldLength: RequireAtLeastOne<FieldLength, 'min' | 'max'>
  scalar: GStringDef

  constructor(scalar: GStringDef, fieldLength: RequireAtLeastOne<FieldLength, 'min' | 'max'>) {
    this.fieldLength = fieldLength
    this.scalar = scalar
  }

  public unique(scope?: string[]): GUniqueDef {
    return new GUniqueDef(this, scope)
  }

  public search(): GSearchDef {
    return new GSearchDef(this)
  }

  public default(val: string): GDefaultDef {
    return new GDefaultDef(this, val)
  }

  public optional(): GLengthLimitedStringDef {
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


export class GNumberDef extends GScalarDef {
  public default(val: number): GDefaultDef {
    return new GDefaultDef(this, val)
  }

  public list(): GNumberListDef {
    return new GNumberListDef(this)
  }
}

export class GBooleanDef extends GScalarDef {
  public default(val: boolean): GDefaultDef {
    return new GDefaultDef(this, val)
  }

  public list(): GBooleanListDef {
    return new GBooleanListDef(this)
  }
}

export class GDateDef extends GScalarDef {
  public default(val: Date): GDefaultDef {
    return new GDefaultDef(this, val)
  }

  public list(): GDateListDef {
    return new GDateListDef(this)
  }
}

export class GObjectDef extends GScalarDef {
  public default(val: object): GDefaultDef {
    return new GDefaultDef(this, val)
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
    switch(fieldType) {
      case FieldType.String: {
        return `"${val}"`
      }
      case FieldType.ID: {
        return `"${val}"`
      }
      case FieldType.Email: {
        return `"${val}"`
      }
      case FieldType.PhoneNumber: {
        return `"${val}"`
      }
      case FieldType.IPAddress: {
        return `"${val}"`
      }
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
