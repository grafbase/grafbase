import { Model, ModelFields } from './model'
import { RelationDefinition, RelationRef } from './relation'
import { Enum, EnumShape } from './enum'
import { Type, TypeFields } from './type'
import { ReferenceDefinition } from './reference'
import { Union } from './union'
import { Interface, InterfaceFields } from './interface'
import { Query, QueryInput } from './query'
import { OpenAPI, PartialOpenAPI } from './connector/openapi'
import { GraphQLAPI, PartialGraphQLAPI } from './connector/graphql'
import {
  BooleanDefinition,
  DateDefinition,
  NumberDefinition,
  ObjectDefinition,
  StringDefinition
} from './typedefs/scalar'
import { FieldType } from './field/typedefs'

export type PartialDatasource = PartialOpenAPI | PartialGraphQLAPI
export type Datasource = OpenAPI | GraphQLAPI

export class Datasources {
  inner: Datasource[]

  constructor() {
    this.inner = []
  }

  push(datasource: Datasource) {
    this.inner.push(datasource)
  }

  public toString(): string {
    if (this.inner.length > 0) {
      const header = 'extend schema'
      const datasources = this.inner.map(String).join('\n')

      return `${header}\n${datasources}`
    } else {
      return ''
    }
  }
}

export interface IntrospectParams {
  namespace: string
}

export class GrafbaseSchema {
  enums: Enum[]
  types: Type[]
  unions: Union[]
  models: Model[]
  interfaces: Interface[]
  queries: Query[]
  mutations: Query[]
  datasources: Datasources

  constructor() {
    this.enums = []
    this.types = []
    this.unions = []
    this.models = []
    this.interfaces = []
    this.queries = []
    this.mutations = []
    this.datasources = new Datasources()
  }

  public datasource(datasource: PartialDatasource, params: IntrospectParams) {
    this.datasources.push(datasource.finalize(params.namespace))
  }

  public model(name: string, fields: ModelFields): Model {
    const model = Object.entries(fields).reduce(
      (model, [name, definition]) => model.field(name, definition),
      new Model(name)
    )

    this.models.push(model)

    return model
  }

  public type(name: string, fields: TypeFields): Type {
    const type = Object.entries(fields).reduce(
      (type, [name, definition]) => type.field(name, definition),
      new Type(name)
    )

    this.types.push(type)

    return type
  }

  public interface(name: string, fields: InterfaceFields): Interface {
    const iface = Object.entries(fields).reduce(
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
        query.argument(name, type)
      )
    }

    this.queries.push(query)

    return query
  }

  public mutation(name: string, definition: QueryInput): Query {
    var query = new Query(name, definition.returns, definition.resolver)

    if (definition.args != null) {
      Object.entries(definition.args).forEach(
        ([name, type]) => query.argument(name, type),
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
    this.datasources = new Datasources()
  }

  public toString(): string {
    var queries = this.queries.map(String).join('\n')
    var mutations = this.mutations.map(String).join('\n')

    queries = queries ? `extend type Query {\n${queries}\n}` : ''
    mutations = mutations ? `extend type Mutation {\n${mutations}\n}` : ''

    const datasources = this.datasources.toString()
    const interfaces = this.interfaces.map(String).join('\n\n')
    const types = this.types.map(String).join('\n\n')
    const unions = this.unions.map(String).join('\n\n')
    const enums = this.enums.map(String).join('\n\n')
    const models = this.models.map(String).join('\n\n')

    const renderOrder = [
      datasources,
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
