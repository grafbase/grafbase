import { ModelFieldShape } from './model'

export class Field {
  name: string
  shape: ModelFieldShape

  constructor(name: string, shape: ModelFieldShape) {
    this.name = name
    this.shape = shape
  }

  public toString(): string {
    return `${this.name}: ${this.shape}`
  }
}
