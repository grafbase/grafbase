import { ScalarType } from '..'
import { ReferenceDef } from '../reference'
import { RelationDef } from '../relation'
import {
  FieldType,
  BooleanDef,
  DateDef,
  NumberDef,
  ScalarDef,
  SearchDef,
  StringDef,
  renderDefault
} from './typedefs'

export class ListDef {
  fieldDefinition: ScalarDef | RelationDef | ReferenceDef
  isOptional: boolean
  defaultValue?: ScalarType[]

  constructor(fieldDefinition: ScalarDef | RelationDef | ReferenceDef) {
    this.fieldDefinition = fieldDefinition
    this.isOptional = false
  }

  public optional(): ListDef {
    this.isOptional = true

    return this
  }

  public search(): SearchDef {
    return new SearchDef(this)
  }

  public toString(): string {
    const required = this.isOptional ? '' : '!'

    return `[${this.fieldDefinition}]${required}`
  }
}

class ListWithDefaultDef extends ListDef {
  fieldType: FieldType

  constructor(fieldDefinition: ScalarDef) {
    super(fieldDefinition)

    this.fieldType = fieldDefinition.fieldType as FieldType
  }

  public toString(): string {
    const defaultValue =
      this.defaultValue != null
        ? ` @default(value: [${this.defaultValue
            .map((v) => renderDefault(v, this.fieldType))
            .join(', ')}])`
        : ''

    return `${super.toString()}${defaultValue}`
  }
}

export class StringListDef extends ListWithDefaultDef {
  constructor(fieldDefinition: StringDef) {
    super(fieldDefinition)
  }

  public default(val: string[]): StringListDef {
    this.defaultValue = val

    return this
  }
}

export class NumberListDef extends ListWithDefaultDef {
  constructor(fieldDefinition: NumberDef) {
    super(fieldDefinition)
  }

  public default(val: number[]): NumberListDef {
    this.defaultValue = val

    return this
  }
}

export class BooleanListDef extends ListWithDefaultDef {
  constructor(fieldDefinition: BooleanDef) {
    super(fieldDefinition)
  }

  public default(val: boolean[]): BooleanListDef {
    this.defaultValue = val

    return this
  }
}

export class DateListDef extends ListWithDefaultDef {
  constructor(fieldDefinition: DateDef) {
    super(fieldDefinition)
  }

  public default(val: Date[]): DateListDef {
    this.defaultValue = val

    return this
  }
}
