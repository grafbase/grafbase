import { Model } from './model'
import { RelationDef } from './relation'
import { Enum } from './enum'
import {
  FieldType,
  BooleanDef,
  DateDef,
  NumberDef,
  ObjectDef,
  StringDef,
  ScalarDef
} from './field/typedefs'
import { ListDef } from './field/list'
import { Type } from './type'
import { ReferenceDef } from './reference'
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
    const model = Object.entries(fields).reduce(
      (model, [name, definition]) => model.field(name, definition),
      new Model(name)
    )

    this.models.push(model)

    return model
  }

  public type(
    name: string,
    fields: Record<string, ScalarDef | ListDef | ReferenceDef>
  ): Type {
    const type = Object.entries(fields).reduce(
      (type, [name, definition]) => type.field(name, definition),
      new Type(name)
    )

    this.types.push(type)

    return type
  }

  public interface(
    name: string,
    types: Record<string, ScalarDef | ListDef>
  ): Interface {
    const iface = Object.entries(types).reduce(
      (iface, [name, definition]) => iface.field(name, definition),
      new Interface(name)
    )

    this.interfaces.push(iface)

    return iface
  }

  public union(name: string, types: Record<string, Type>): Union {
    const union = Object.entries(types).reduce(
      (model, [_, type]) => model.type(type),
      new Union(name)
    )

    this.unions.push(union)

    return union
  }

  public query(name: string, definition: QueryInput): Query {
    var query = new Query(
      name,
      QueryType.Query,
      definition.returns,
      definition.resolver
    )

    if (definition.args != null) {
      Object.entries(definition.args).forEach(([name, type]) =>
        query.pushArgument(name, type)
      )
    }

    this.queries.push(query)

    return query
  }

  public mutation(name: string, definition: QueryInput): Query {
    var query = new Query(
      name,
      QueryType.Mutation,
      definition.returns,
      definition.resolver
    )

    if (definition.args != null) {
      Object.entries(definition.args).forEach(
        ([name, type]) => query.pushArgument(name, type),
        query
      )
    }

    this.queries.push(query)

    return query
  }

  public enum(name: string, variants: EnumShape): Enum {
    const e = new Enum(name, variants)

    this.enums.push(e)

    return e
  }

  public string(): StringDef {
    return new StringDef(FieldType.String)
  }

  public id(): StringDef {
    return new StringDef(FieldType.ID)
  }

  public email(): StringDef {
    return new StringDef(FieldType.Email)
  }

  public int(): NumberDef {
    return new NumberDef(FieldType.Int)
  }

  public float(): NumberDef {
    return new NumberDef(FieldType.Float)
  }

  public boolean(): BooleanDef {
    return new BooleanDef(FieldType.Boolean)
  }

  public date(): DateDef {
    return new DateDef(FieldType.Date)
  }

  public datetime(): DateDef {
    return new DateDef(FieldType.DateTime)
  }

  public ipAddress(): StringDef {
    return new StringDef(FieldType.IPAddress)
  }

  public timestamp(): NumberDef {
    return new NumberDef(FieldType.Timestamp)
  }

  public url(): StringDef {
    return new StringDef(FieldType.URL)
  }

  public json(): ObjectDef {
    return new ObjectDef(FieldType.JSON)
  }

  public phoneNumber(): StringDef {
    return new StringDef(FieldType.PhoneNumber)
  }

  public relation(ref: RelationRef): RelationDef {
    return new RelationDef(ref)
  }

  public ref(type: Type | Enum): ReferenceDef {
    return new ReferenceDef(type)
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
    const queries = this.queries.map(String).join('\n\n')
    const interfaces = this.interfaces.map(String).join('\n\n')
    const types = this.types.map(String).join('\n\n')
    const unions = this.unions.map(String).join('\n\n')
    const enums = this.enums.map(String).join('\n\n')
    const models = this.models.map(String).join('\n\n')

    const renderOrder = [interfaces, enums, types, queries, unions, models]

    return renderOrder.filter(Boolean).flat().map(String).join('\n\n')
  }
}
