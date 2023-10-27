import { FieldShape as DynamoFieldShape } from './connector/dynamodb/model'
import { FieldShape as MongoFieldShape } from './connector/mongodb/model'
import { JoinDefinition } from './typedefs/join'
import { validateIdentifier } from './validation'

type FieldShape = DynamoFieldShape | MongoFieldShape | JoinDefinition

export class Field {
  private _name: string
  private shape: FieldShape

  constructor(name: string, shape: FieldShape) {
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
