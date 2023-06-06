import { ModelFieldShape } from './model'
import { validateIdentifier } from './validation'

export class Field {
  private _name: string
  private shape: ModelFieldShape

  constructor(name: string, shape: ModelFieldShape) {
    validateIdentifier(name)

    this._name = name
    this.shape = shape
  }

  public get name(): string {
    return this._name
  }

  public toString(): string {
    return `${this.name}: ${this.shape}`
  }
}
