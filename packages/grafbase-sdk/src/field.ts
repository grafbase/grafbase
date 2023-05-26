import { ModelFieldShape } from './model'
import { validateIdentifier } from './validation'

export class Field {
  name: string
  shape: ModelFieldShape

  constructor(name: string, shape: ModelFieldShape) {
    validateIdentifier(name)

    this.name = name
    this.shape = shape
  }

  public toString(): string {
    return `${this.name}: ${this.shape}`
  }
}
