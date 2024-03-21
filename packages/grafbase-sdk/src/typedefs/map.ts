import { InputType } from '../query'
import { AuthDefinition } from './auth'
import { CacheDefinition } from './cache'
import { DefaultDefinition } from './default'
import { EnumDefinition } from './enum'
import { LengthLimitedStringDefinition } from './length-limited-string'
import { ListDefinition } from './list'
import { ReferenceDefinition } from './reference'
import { ScalarDefinition } from './scalar'
import { UniqueDefinition } from './unique'

export type Mappable =
  | ScalarDefinition
  | DefaultDefinition
  | ReferenceDefinition
  | UniqueDefinition
  | LengthLimitedStringDefinition
  | AuthDefinition
  | CacheDefinition
  | ListDefinition
  | EnumDefinition<any, any>

export class MapDefinition {
  private field: Mappable
  private mappedName: string

  constructor(field: Mappable, mappedName: string) {
    this.field = field
    this.mappedName = mappedName
  }

  public get allArguments(): Record<string, InputType> {
    return { ...this.field.allArguments }
  }

  public toString(): string {
    return `${this.field} @map(name: "${this.mappedName}")`
  }
}
