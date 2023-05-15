import { GListDef } from './field/list'
import {
  GDefaultDef,
  GLengthLimitedStringDef,
  GScalarDef,
  GSearchDef,
  GUniqueDef
} from './field/typedefs'
import { Model } from './model'
import { GReferenceDef } from './reference'
import { GRelationDef } from './relation'
import { GrafbaseSchema } from './grafbase-schema'
import { Config, ConfigInput } from './config'

export type FieldShape =
  | GScalarDef
  | GRelationDef
  | GListDef
  | GSearchDef
  | GReferenceDef
  | GUniqueDef
  | GDefaultDef
  | GLengthLimitedStringDef

export type AtLeastOne<T> = [T, ...T[]]
export type ScalarType = string | number | Date | object | boolean
export type EnumShape = AtLeastOne<string> | { [s: number]: string }
export type RelationRef = RelationF | Model
export type Searchable = GScalarDef | GListDef | GUniqueDef

type RelationF = () => Model

export const g = new GrafbaseSchema()

export function config(input: ConfigInput): Config {
  return new Config(input)
}
