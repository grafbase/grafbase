import { Enum } from "./enum";
import { GListDef } from "./field/list";
import { Type } from "./type";

export class GReferenceDef {
  referencedType: string
  isOptional: boolean

  constructor(referencedType: Type | Enum) {
    this.referencedType = referencedType.name
    this.isOptional = false
  }
  
  public optional(): GReferenceDef {
    this.isOptional = true

    return this
  }

  public list(): GListDef {
    return new GListDef(this)
  }

  public toString(): string {
    const required = this.isOptional ? "" : "!"

    return `${this.referencedType}${required}`
  }
}