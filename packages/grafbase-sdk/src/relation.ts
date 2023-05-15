import { RelationRef } from '.'
import { ListDef } from './field/list'

export class RelationDef {
  relationName?: string
  referencedModel: RelationRef
  isOptional: boolean

  constructor(referencedModel: RelationRef) {
    this.referencedModel = referencedModel
    this.isOptional = false
  }

  public optional(): RelationDef {
    this.isOptional = true

    return this
  }

  public list(): ListDef {
    return new ListDef(this)
  }

  public name(name: string): RelationDef {
    this.relationName = name

    return this
  }

  public toString(): string {
    let modelName

    if (typeof this.referencedModel === 'function') {
      modelName = this.referencedModel().name
    } else {
      modelName = this.referencedModel.name
    }

    const required = this.isOptional ? '' : '!'
    const relationAttribute = this.relationName
      ? ` @relation(name: ${this.relationName})`
      : ''

    return `${modelName}${required}${relationAttribute}`
  }
}
