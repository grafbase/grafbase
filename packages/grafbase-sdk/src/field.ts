import { FieldShape } from '.'

export class Field {
  name: string
  shape: FieldShape

  constructor(name: string, shape: FieldShape) {
    this.name = name
    this.shape = shape
  }

  public toString(): string {
    return `${this.name}: ${this.shape}`
  }
}
