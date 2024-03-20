import { FieldShape as MongoFieldShape } from './connector/mongodb/model'
import { FieldArgument } from './query'
import { AuthDefinition } from './typedefs/auth'
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
  | AuthDefinition

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
    const args = Object.entries(this.shape.allArguments)
      .map(([name, ty]) => new FieldArgument(name, ty).toString())
      .join(', ')

    const argsStr = args ? `(${args})` : ''
    return `${this.name}${argsStr}: ${this.shape}`
  }
}
