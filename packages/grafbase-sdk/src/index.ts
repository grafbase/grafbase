import { Model } from './model'
import { Config } from './config'
import { GRelationDef } from './relation'
import { Enum } from './enum'
import { FieldType, GBooleanDef, GDateDef, GNumberDef, GObjectDef, GStringDef, GScalarDef, GSearchDef, GUniqueDef, GDefaultDef, GLengthLimitedStringDef } from './field/typedefs'
import { GListDef } from './field/list'
import { Type } from './type'
import { GReferenceDef } from './reference'
import { Union } from './union'
import { Interface } from './interface'
import { PartialQuery, QueryInput, QueryType } from './query'

export type AtLeastOne<T> = [T, ...T[]]
export type ScalarType = string | number | Date | object | boolean;
export type FieldShape = GScalarDef | GRelationDef | GListDef | GSearchDef | GReferenceDef | GUniqueDef | GDefaultDef | GLengthLimitedStringDef
export type EnumShape = AtLeastOne<string> | { [s: number]: string }
export type RelationRef = RelationF | Model
export type Searchable = GScalarDef | GListDef | GUniqueDef

type RelationF = () => Model

export const g = {
  model: function(name: string, fields: Record<string, FieldShape>): Model {
    return Object
      .entries(fields)
      .reduce((model, [name, definition]) => model.field(name, definition), new Model(name))
  },

  type: function(name: string, fields: Record<string, GScalarDef | GListDef>): Type {
    return Object
      .entries(fields)
      .reduce((type, [name, definition]) => type.field(name, definition), new Type(name))
  },

  interface: function(name: string, types: Record<string, GScalarDef | GListDef>): Interface {
    return Object
      .entries(types)
      .reduce((iface, [name, definition]) => iface.field(name, definition), new Interface(name))
  },

  union: function(name: string, types: Record<string, Type>): Union {
    return Object
      .entries(types)
      .reduce((model, [_, type]) => model.type(type), new Union(name))
  },

  query: function(name: string, definition: QueryInput): PartialQuery {
    const q = new PartialQuery(name, QueryType.Query, definition.returns)

    return Object
      .entries(definition.args)
      .reduce((query, [name, type]) => query.argument(name, type), q)
  },

  mutation: function(name: string, definition: QueryInput): PartialQuery {
    const q = new PartialQuery(name, QueryType.Mutation, definition.returns)

    return Object
      .entries(definition.args)
      .reduce((query, [name, type]) => query.argument(name, type), q)
  },

  enumType: function(name: string, variants: EnumShape): Enum {
    return new Enum(name, variants)
  },

  enum: function(e: Enum): GStringDef {
    return new GStringDef(e)
  },

  string: function(): GStringDef {
    return new GStringDef(FieldType.String)
  },

  id: function(): GStringDef {
    return new GStringDef(FieldType.ID)
  },

  email: function(): GStringDef {
    return new GStringDef(FieldType.Email)
  },

  int: function(): GNumberDef {
    return new GNumberDef(FieldType.Int)
  },

  float: function(): GNumberDef {
    return new GNumberDef(FieldType.Float)
  },

  boolean: function(): GBooleanDef {
    return new GBooleanDef(FieldType.Boolean)
  },

  date: function(): GDateDef {
    return new GDateDef(FieldType.Date)
  },

  datetime: function(): GDateDef {
    return new GDateDef(FieldType.DateTime)
  },

  ipAddress: function(): GStringDef {
    return new GStringDef(FieldType.IPAddress)
  },

  timestamp: function(): GNumberDef {
    return new GNumberDef(FieldType.Timestamp)
  },

  url: function(): GStringDef {
    return new GStringDef(FieldType.URL)
  },

  json: function(): GObjectDef {
    return new GObjectDef(FieldType.JSON)
  },

  phoneNumber: function(): GStringDef {
    return new GStringDef(FieldType.PhoneNumber)
  },

  relation: function(ref: RelationRef): GRelationDef {
    return new GRelationDef(ref)
  },

  ref: function(ref: Type): GReferenceDef {
    return new GReferenceDef(ref)
  }
}

export function config(): Config {
  return new Config()
}