import { Model } from './model'
import { GRelationDef } from './relation'
import { Enum } from './enum'
import { FieldType, GBooleanDef, GDateDef, GNumberDef, GObjectDef, GStringDef, GScalarDef } from './field/typedefs'
import { GListDef } from './field/list'
import { Type } from './type'
import { GReferenceDef } from './reference'
import { Union } from './union'
import { Interface } from './interface'
import { Query, QueryInput, QueryType } from './query'
import { EnumShape, FieldShape, RelationRef } from '.'

export class GrafbaseSchema {
  enums: Enum[]
  types: Type[]
  unions: Union[]
  models: Model[]
  interfaces: Interface[]
  queries: Query[]

  constructor() {
    this.enums = []
    this.types = []
    this.unions = []
    this.models = []
    this.interfaces = []
    this.queries = []
  }

  public model(name: string, fields: Record<string, FieldShape>): Model {
    const model = Object
      .entries(fields)
      .reduce((model, [name, definition]) => model.field(name, definition), new Model(name))

    this.models.push(model)

    return model
  }

  public type(name: string, fields: Record<string, GScalarDef | GListDef>): Type {
    const type = Object
      .entries(fields)
      .reduce((type, [name, definition]) => type.field(name, definition), new Type(name))

    this.types.push(type)

    return type
  }

  public interface(name: string, types: Record<string, GScalarDef | GListDef>): Interface {
    const iface = Object
      .entries(types)
      .reduce((iface, [name, definition]) => iface.field(name, definition), new Interface(name))

    this.interfaces.push(iface)

    return iface
  }

  public union(name: string, types: Record<string, Type>): Union {
    const union = Object
      .entries(types)
      .reduce((model, [_, type]) => model.type(type), new Union(name))

    this.unions.push(union)

    return union
  }

  public query(name: string, definition: QueryInput): Query {
    const q = new Query(name, QueryType.Query, definition.returns, definition.resolver)

    let query
    if (definition.args != null) {
      query = Object
        .entries(definition.args)
        .reduce((query, [name, type]) => query.pushArgument(name, type), q)
    } else {
      query = q
    }

    this.queries.push(query)

    return query
  }

  public mutation(name: string, definition: QueryInput): Query {
    const q = new Query(name, QueryType.Mutation, definition.returns, definition.resolver)

    let query
    if (definition.args != null) {
      query = Object
        .entries(definition.args)
        .reduce((query, [name, type]) => query.pushArgument(name, type), q)
    } else {
      query = q
    }

    this.queries.push(query)

    return query
  }

  public enum(name: string, variants: EnumShape): Enum {
    const e = new Enum(name, variants)

    this.enums.push(e)

    return e
  }

  public string(): GStringDef {
    return new GStringDef(FieldType.String)
  }

  public id(): GStringDef {
    return new GStringDef(FieldType.ID)
  }

  public email(): GStringDef {
    return new GStringDef(FieldType.Email)
  }

  public int(): GNumberDef {
    return new GNumberDef(FieldType.Int)
  }

  public float(): GNumberDef {
    return new GNumberDef(FieldType.Float)
  }

  public boolean(): GBooleanDef {
    return new GBooleanDef(FieldType.Boolean)
  }

  public date(): GDateDef {
    return new GDateDef(FieldType.Date)
  }

  public datetime(): GDateDef {
    return new GDateDef(FieldType.DateTime)
  }

  public ipAddress(): GStringDef {
    return new GStringDef(FieldType.IPAddress)
  }

  public timestamp(): GNumberDef {
    return new GNumberDef(FieldType.Timestamp)
  }

  public url(): GStringDef {
    return new GStringDef(FieldType.URL)
  }

  public json(): GObjectDef {
    return new GObjectDef(FieldType.JSON)
  }

  public phoneNumber(): GStringDef {
    return new GStringDef(FieldType.PhoneNumber)
  }

  public relation(ref: RelationRef): GRelationDef {
    return new GRelationDef(ref)
  }

  public ref(type: Type | Enum): GReferenceDef {
    return new GReferenceDef(type)
  }

  public clear() {
    this.queries = []
    this.interfaces = []
    this.types = []
    this.unions = []
    this.enums = []
    this.models = []
  }

  public toString(): string {
    const queries = this.queries.map(String).join("\n\n")
    const interfaces = this.interfaces.map(String).join("\n\n")
    const types = this.types.map(String).join("\n\n")
    const unions = this.unions.map(String).join("\n\n")
    const enums = this.enums.map(String).join("\n\n")
    const models = this.models.map(String).join("\n\n")

    const renderOrder = [interfaces, enums, types, queries, unions, models]

    return renderOrder.filter(Boolean).flat().map(String).join("\n\n")
  }
}