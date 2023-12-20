import { FieldShape as MongoFieldShape } from './connector/mongodb/model'
import { DeprecatedDefinition } from './typedefs/deprecated'
import { InaccessibleDefinition } from './typedefs/inaccessible'
import { JoinDefinition } from './typedefs/join'
import { OverrideDefinition } from './typedefs/override'
import { ProvidesDefinition } from './typedefs/provides'
import { ShareableDefinition } from './typedefs/shareable'
import { TagDefinition } from './typedefs/tag'
import { validateIdentifier } from './validation'

type FieldShape =
  | MongoFieldShape
  | JoinDefinition
  | TagDefinition
  | InaccessibleDefinition
  | ShareableDefinition
  | OverrideDefinition
  | ProvidesDefinition
  | DeprecatedDefinition

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
