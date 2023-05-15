import { ListDef } from './field/list'
import {
  DefaultDef,
  LengthLimitedStringDef,
  ScalarDef,
  SearchDef,
  UniqueDef
} from './field/typedefs'
import { Model } from './model'
import { ReferenceDef } from './reference'
import { RelationDef } from './relation'
import { GrafbaseSchema } from './grafbase-schema'
import { Config, ConfigInput } from './config'

export type FieldShape =
  | ScalarDef
  | RelationDef
  | ListDef
  | SearchDef
  | ReferenceDef
  | UniqueDef
  | DefaultDef
  | LengthLimitedStringDef

export type AtLeastOne<T> = [T, ...T[]]
export type ScalarType = string | number | Date | object | boolean
export type EnumShape = AtLeastOne<string> | { [s: number]: string }
export type RelationRef = RelationF | Model
export type Searchable = ScalarDef | ListDef | UniqueDef

type RelationF = () => Model

export const g = new GrafbaseSchema()

export function config(input: ConfigInput): Config {
  return new Config(input)
}
