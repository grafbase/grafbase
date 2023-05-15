import { RelationRef } from '.'
import { ListDefinition } from './field/list'

export class RelationDefinition {
  relationName?: string
  referencedModel: RelationRef
  isOptional: boolean

  constructor(referencedModel: RelationRef) {
    this.referencedModel = referencedModel
    this.isOptional = false
  }

  public optional(): RelationDefinition {
    this.isOptional = true

    return this
  }

  public list(): ListDefinition {
    return new ListDefinition(this)
  }

  public name(name: string): RelationDefinition {
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
