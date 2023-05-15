import { RelationRef } from '.'
import { GListDef } from './field/list'

export class GRelationDef {
  relationName?: string
  referencedModel: RelationRef
  isOptional: boolean

  constructor(referencedModel: RelationRef) {
    this.referencedModel = referencedModel
    this.isOptional = false
  }

  public optional(): GRelationDef {
    this.isOptional = true

    return this
  }

  public list(): GListDef {
    return new GListDef(this)
  }

  public name(name: string): GRelationDef {
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
