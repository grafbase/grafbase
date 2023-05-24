import { Model, ModelFields } from './model'
import { RelationDefinition, RelationRef } from './relation'
import { Enum, EnumShape } from './enum'
import { Type, TypeFields } from './type'
import { ReferenceDefinition } from './typedefs/reference'
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
import { FieldType } from './typedefs'
import { EnumDefinition } from './typedefs/enum'

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
  enums: Enum<any, any>[]
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

  /**
   * Adds a new datasource to the schema.
   */
  public datasource(datasource: PartialDatasource, params: IntrospectParams) {
    this.datasources.push(datasource.finalize(params.namespace))
  }

  /**
   * Adds a new model to the schema.
   */
  public model(name: string, fields: ModelFields): Model {
    const model = Object.entries(fields).reduce(
      (model, [name, definition]) => model.field(name, definition),
      new Model(name)
    )

    this.models.push(model)

    return model
  }

  /**
   * Adds a new composite type to the schema.
   */
  public type(name: string, fields: TypeFields): Type {
    const type = Object.entries(fields).reduce(
      (type, [name, definition]) => type.field(name, definition),
      new Type(name)
    )

    this.types.push(type)

    return type
  }

  /**
   * Adds a new interface to the schema.
   */
  public interface(name: string, fields: InterfaceFields): Interface {
    const iface = Object.entries(fields).reduce(
      (iface, [name, definition]) => iface.field(name, definition),
      new Interface(name)
    )

    this.interfaces.push(iface)

    return iface
  }

  /**
   * Adds a new union to the schema.
   */
  public union(name: string, types: Record<string, Type>): Union {
    const union = Object.entries(types).reduce(
      (model, [_, type]) => model.type(type),
      new Union(name)
    )

    this.unions.push(union)

    return union
  }

  /**
   * Adds a new query to the schema.
   */
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

  /**
   * Adds a new mutation to the schema.
   */
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

  /**
   * Adds a new enum to the schema.
   */
  public enum<T extends string, U extends EnumShape<T>>(
    name: string,
    variants: U
  ): Enum<T, U> {
    const e = new Enum(name, variants)
    this.enums.push(e)

    return e
  }

  /**
   * Creates a new string field.
   */
  public string(): StringDefinition {
    return new StringDefinition(FieldType.String)
  }

  /**
   * Creates a new ID field.
   */
  public id(): StringDefinition {
    return new StringDefinition(FieldType.ID)
  }

  /**
   * Creates a new email field.
   */
  public email(): StringDefinition {
    return new StringDefinition(FieldType.Email)
  }

  /**
   * Creates a new int field.
   */
  public int(): NumberDefinition {
    return new NumberDefinition(FieldType.Int)
  }

  /**
   * Creates a new float field.
   */
  public float(): NumberDefinition {
    return new NumberDefinition(FieldType.Float)
  }

  /**
   * Creates a new boolean field.
   */
  public boolean(): BooleanDefinition {
    return new BooleanDefinition(FieldType.Boolean)
  }

  /**
   * Creates a new date field.
   */
  public date(): DateDefinition {
    return new DateDefinition(FieldType.Date)
  }

  /**
   * Creates a new datetime field.
   */
  public datetime(): DateDefinition {
    return new DateDefinition(FieldType.DateTime)
  }

  /**
   * Creates a new IP address field.
   */
  public ipAddress(): StringDefinition {
    return new StringDefinition(FieldType.IPAddress)
  }

  /**
   * Creates a new timestamp field.
   */
  public timestamp(): NumberDefinition {
    return new NumberDefinition(FieldType.Timestamp)
  }

  /**
   * Creates a new URL field.
   */
  public url(): StringDefinition {
    return new StringDefinition(FieldType.URL)
  }

  /**
   * Creates a new JSON field.
   */
  public json(): ObjectDefinition {
    return new ObjectDefinition(FieldType.JSON)
  }

  /**
   * Creates a new phone number field.
   */
  public phoneNumber(): StringDefinition {
    return new StringDefinition(FieldType.PhoneNumber)
  }

  /**
   * Creates a new relation field.
   */
  public relation(ref: RelationRef): RelationDefinition {
    return new RelationDefinition(ref)
  }

  /**
   * Creates a new reference field, referencing a type.
   */
  public ref(type: Type): ReferenceDefinition {
    return new ReferenceDefinition(type)
  }

  /**
   * Creates a new enum field.
   */
  public enumRef<T extends string, U extends EnumShape<T>>(
    e: Enum<T, U>
  ): EnumDefinition<T, U> {
    return new EnumDefinition(e)
  }

  /**
   * Empties the schema.
   */
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
