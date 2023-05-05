import { ScalarType } from ".."
import { GReferenceDef } from "../reference"
import { GRelationDef } from "../relation"
import { FieldType, GBooleanDef, GDateDef, GNumberDef, GScalarDef, GSearchDef, GStringDef, renderDefault } from "./typedefs"

export class GListDef {
  fieldDefinition: GScalarDef | GRelationDef | GReferenceDef
  isOptional: boolean
  defaultValue?: ScalarType[]
  
  constructor(fieldDefinition: GScalarDef | GRelationDef | GReferenceDef) {
    this.fieldDefinition = fieldDefinition
    this.isOptional = false
  }

  public optional(): GListDef {
    this.isOptional = true

    return this
  }

  public search(): GSearchDef {
    return new GSearchDef(this)
  }

  public toString(): string {
    const required = this.isOptional ? "" : "!"

    return `[${this.fieldDefinition}]${required}`
  }
}

class GListWithDefaultDef extends GListDef {
  fieldType: FieldType

  constructor(fieldDefinition: GScalarDef) {
    super(fieldDefinition)

    this.fieldType = fieldDefinition.fieldType as FieldType
  }

  public toString(): string {
    const defaultValue = this.defaultValue != null ?
      ` @default(value: [${this.defaultValue.map((v) => renderDefault(v, this.fieldType)).join(', ')}])` : 
      ""

    return `${super.toString()}${defaultValue}`
  }
}

export class GStringListDef extends GListWithDefaultDef {
  constructor(fieldDefinition: GStringDef) {
    super(fieldDefinition)
  }

  public default(val: string[]): GStringListDef {
    this.defaultValue = val

    return this
  }
}

export class GNumberListDef extends GListWithDefaultDef {
  constructor(fieldDefinition: GNumberDef) {
    super(fieldDefinition)
  }

  public default(val: number[]): GNumberListDef {
    this.defaultValue = val

    return this
  }
}

export class GBooleanListDef extends GListWithDefaultDef {
  constructor(fieldDefinition: GBooleanDef) {
    super(fieldDefinition)
  }

  public default(val: boolean[]): GBooleanListDef {
    this.defaultValue = val

    return this
  }
}

export class GDateListDef extends GListWithDefaultDef {
  constructor(fieldDefinition: GDateDef) {
    super(fieldDefinition)
  }

  public default(val: Date[]): GDateListDef {
    this.defaultValue = val

    return this
  }
}
