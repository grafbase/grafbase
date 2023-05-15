import { Enum } from './enum'
import { ListDef } from './field/list'
import { Type } from './type'

export class ReferenceDef {
  referencedType: string
  isOptional: boolean

  constructor(referencedType: Type | Enum) {
    this.referencedType = referencedType.name
    this.isOptional = false
  }

  public optional(): ReferenceDef {
    this.isOptional = true

    return this
  }

  public list(): ListDef {
    return new ListDef(this)
  }

  public toString(): string {
    const required = this.isOptional ? '' : '!'

    return `${this.referencedType}${required}`
  }
}
