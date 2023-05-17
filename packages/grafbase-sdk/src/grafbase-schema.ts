import { Model } from './model'
import { RelationDefinition } from './relation'
import { Enum } from './enum'
import {
  FieldType,
  BooleanDefinition,
  DateDefinition,
  NumberDefinition,
  ObjectDefinition,
  StringDefinition,
  ScalarDefinition
} from './field/typedefs'
import { ListDefinition } from './field/list'
import { Type } from './type'
import { ReferenceDefinition } from './reference'
import { Union } from './union'
import { Interface } from './interface'
import { Query, QueryInput } from './query'
import { EnumShape, FieldShape, RelationRef } from '.'

export class GrafbaseSchema {
  enums: Enum[]
  types: Type[]
  unions: Union[]
  models: Model[]
  interfaces: Interface[]
  queries: Query[]
  mutations: Query[]

  constructor() {
    this.enums = []
    this.types = []
    this.unions = []
    this.models = []
    this.interfaces = []
    this.queries = []
    this.mutations = []
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
    fields: Record<
      string,
      ScalarDefinition | ListDefinition | ReferenceDefinition
    >
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
    types: Record<string, ScalarDefinition | ListDefinition>
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
    var query = new Query(name, definition.returns, definition.resolver)

    if (definition.args != null) {
      Object.entries(definition.args).forEach(([name, type]) =>
        query.pushArgument(name, type)
      )
    }

    this.queries.push(query)

    return query
  }

  public mutation(name: string, definition: QueryInput): Query {
    var query = new Query(name, definition.returns, definition.resolver)

    if (definition.args != null) {
      Object.entries(definition.args).forEach(
        ([name, type]) => query.pushArgument(name, type),
        query
      )
    }

    this.mutations.push(query)

    return query
  }

  public enum(name: string, variants: EnumShape): Enum {
    const e = new Enum(name, variants)

    this.enums.push(e)

    return e
  }

  public string(): StringDefinition {
    return new StringDefinition(FieldType.String)
  }

  public id(): StringDefinition {
    return new StringDefinition(FieldType.ID)
  }

  public email(): StringDefinition {
    return new StringDefinition(FieldType.Email)
  }

  public int(): NumberDefinition {
    return new NumberDefinition(FieldType.Int)
  }

  public float(): NumberDefinition {
    return new NumberDefinition(FieldType.Float)
  }

  public boolean(): BooleanDefinition {
    return new BooleanDefinition(FieldType.Boolean)
  }

  public date(): DateDefinition {
    return new DateDefinition(FieldType.Date)
  }

  public datetime(): DateDefinition {
    return new DateDefinition(FieldType.DateTime)
  }

  public ipAddress(): StringDefinition {
    return new StringDefinition(FieldType.IPAddress)
  }

  public timestamp(): NumberDefinition {
    return new NumberDefinition(FieldType.Timestamp)
  }

  public url(): StringDefinition {
    return new StringDefinition(FieldType.URL)
  }

  public json(): ObjectDefinition {
    return new ObjectDefinition(FieldType.JSON)
  }

  public phoneNumber(): StringDefinition {
    return new StringDefinition(FieldType.PhoneNumber)
  }

  public relation(ref: RelationRef): RelationDefinition {
    return new RelationDefinition(ref)
  }

  public ref(type: Type | Enum): ReferenceDefinition {
    return new ReferenceDefinition(type)
  }

  public clear() {
    this.queries = []
    this.mutations = []
    this.interfaces = []
    this.types = []
    this.unions = []
    this.enums = []
    this.models = []
  }

  public toString(): string {
    var queries = this.queries.map(String).join('\n')
    var mutations = this.mutations.map(String).join('\n')

    queries = queries ? `extend type Query {\n${queries}\n}` : ''
    mutations = mutations ? `extend type Mutation {\n${mutations}\n}` : ''

    const interfaces = this.interfaces.map(String).join('\n\n')
    const types = this.types.map(String).join('\n\n')
    const unions = this.unions.map(String).join('\n\n')
    const enums = this.enums.map(String).join('\n\n')
    const models = this.models.map(String).join('\n\n')

    const renderOrder = [
      interfaces,
      enums,
      types,
      queries,
      mutations,
      unions,
      models
    ]

    return renderOrder.filter(Boolean).flat().map(String).join('\n\n')
  }
}
