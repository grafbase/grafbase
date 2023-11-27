import { Model } from './model'
import { RelationDefinition, RelationRef } from './relation'
import { Enum, EnumShape } from './enum'
import { Type, TypeExtension, TypeFields } from './type'
import { ReferenceDefinition } from './typedefs/reference'
import { Union } from './union'
import { Interface, InterfaceFields } from './interface'
import { Query, QueryInput } from './query'
import { OpenAPI, PartialOpenAPI } from './connector/openapi'
import { GraphQLAPI, PartialGraphQLAPI } from './connector/graphql'
import {
  BigIntDefinition,
  BooleanDefinition,
  BytesDefinition,
  DateDefinition,
  NumberDefinition,
  ObjectDefinition,
  StringDefinition
} from './typedefs/scalar'
import { FieldType } from './typedefs'
import { EnumDefinition } from './typedefs/enum'
import { Input, InputFields } from './input_type'
import { InputDefinition } from './typedefs/input'
import { MongoDBAPI, PartialMongoDBAPI } from './connector/mongodb'
import { DynamoDBModel, ModelFields } from './connector/dynamodb/model'
import { PostgresAPI, PartialPostgresAPI } from './connector/postgres'
import { Federation } from './federation'

export type PartialDatasource =
  | PartialOpenAPI
  | PartialGraphQLAPI
  | PartialMongoDBAPI
  | PartialPostgresAPI

export type Datasource = OpenAPI | GraphQLAPI | MongoDBAPI | PostgresAPI

export class Datasources {
  inner: Datasource[]

  constructor() {
    this.inner = []
  }

  push(datasource: Datasource) {
    this.inner.push(datasource)
  }

  public toString(): string {
    return this.inner.map(String).join('\n')
  }
}

export interface IntrospectParams {
  namespace?: boolean
}

export class SingleGraph {
  private enums: Enum<any, any>[]
  private types: Type[]
  private unions: Union[]
  private models: Model[]
  private interfaces: Interface[]
  private queries?: TypeExtension
  private mutations?: TypeExtension
  private datasources: Datasources
  private extendedTypes: TypeExtension[]
  private inputs: Input[]
  federation?: Federation

  constructor() {
    this.enums = []
    this.types = []
    this.unions = []
    this.models = []
    this.interfaces = []
    this.datasources = new Datasources()
    this.extendedTypes = []
    this.inputs = []
  }

  /**
   * Add a new datasource to the schema.
   *
   * @param datasource - The datasource to add.
   * @param params - The introspection parameters.
   */
  public datasource(datasource: PartialDatasource, params?: IntrospectParams) {
    const finalDatasource = datasource.finalize(params?.namespace)

    this.datasources.push(finalDatasource)
  }

  /**
   * Add a new model to the schema.
   *
   * @deprecated The Grafbase database is deprecated and will be sunset soon. Use connectors like Postgres or MongoDB instead.
   *
   * @param name - The name of the model.
   * @param fields - The fields to be included.
   */
  public model(name: string, fields: ModelFields): DynamoDBModel {
    const model = Object.entries(fields).reduce(
      (model, [name, definition]) => model.field(name, definition),
      new DynamoDBModel(name)
    )

    this.models.push(model)

    return model
  }

  /**
   * Add a new composite type to the schema.
   *
   * @param name - The name of the type.
   * @param fields - The fields to be included.
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
   * Add a new interface to the schema.
   *
   * @param name - The name of the interface.
   * @param fields - The fields to be included.
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
   * Add a new union to the schema.
   *
   * @param name - The name of the union.
   * @param types - The types to be included.
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
   * Add a new query to the schema.
   *
   * @param name - The name of the query.
   * @param definition - The query definition.
   */
  public query(name: string, definition: QueryInput): Query {
    const query = new Query(name, definition.returns, definition.resolver)

    if (definition.args != null) {
      Object.entries(definition.args).forEach(([name, type]) =>
        query.argument(name, type)
      )
    }

    if (!this.queries) {
      this.queries = new TypeExtension('Query')
    }

    this.queries.query(query)

    return query
  }

  /**
   * Add a new mutation to the schema.
   *
   * @param name - The name of the mutation.
   * @param fields - The mutation definition.
   */
  public mutation(name: string, definition: QueryInput): Query {
    const query = new Query(name, definition.returns, definition.resolver)

    if (definition.args != null) {
      Object.entries(definition.args).forEach(
        ([name, type]) => query.argument(name, type),
        query
      )
    }

    if (!this.mutations) {
      this.mutations = new TypeExtension('Mutation')
    }

    this.mutations.query(query)

    return query
  }

  /**
   * Add a new input to the schema.
   *
   * @param name = The name of the input.
   * @param fields = The input definition.
   */
  public input(name: string, definition: InputFields): Input {
    const input = new Input(name)

    Object.entries(definition).forEach(([name, type]) => {
      input.field(name, type)
    })

    this.inputs.push(input)

    return input
  }

  /**
   * Add a new enum to the schema.
   *
   * @param name - The name of the enum.
   * @param variants - A list of variants of the enum.
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
   * Create a new string field.
   */
  public string(): StringDefinition {
    return new StringDefinition(FieldType.String)
  }

  /**
   * Create a new ID field.
   */
  public id(): StringDefinition {
    return new StringDefinition(FieldType.ID)
  }

  /**
   * Create a new email field.
   */
  public email(): StringDefinition {
    return new StringDefinition(FieldType.Email)
  }

  /**
   * Create a new int field.
   */
  public int(): NumberDefinition {
    return new NumberDefinition(FieldType.Int)
  }

  /**
   * Create a new float field.
   */
  public float(): NumberDefinition {
    return new NumberDefinition(FieldType.Float)
  }

  /**
   * Create a new boolean field.
   */
  public boolean(): BooleanDefinition {
    return new BooleanDefinition(FieldType.Boolean)
  }

  /**
   * Create a new date field.
   */
  public date(): DateDefinition {
    return new DateDefinition(FieldType.Date)
  }

  /**
   * Create a new datetime field.
   */
  public datetime(): DateDefinition {
    return new DateDefinition(FieldType.DateTime)
  }

  /**
   * Create a new IP address field.
   */
  public ipAddress(): StringDefinition {
    return new StringDefinition(FieldType.IPAddress)
  }

  /**
   * Create a new timestamp field.
   */
  public timestamp(): NumberDefinition {
    return new NumberDefinition(FieldType.Timestamp)
  }

  /**
   * Create a new URL field.
   */
  public url(): StringDefinition {
    return new StringDefinition(FieldType.URL)
  }

  /**
   * Create a new JSON field.
   */
  public json(): ObjectDefinition {
    return new ObjectDefinition(FieldType.JSON)
  }

  /**
   * Create a new phone number field.
   */
  public phoneNumber(): StringDefinition {
    return new StringDefinition(FieldType.PhoneNumber)
  }

  /**
   * Create a new decimal field.
   */
  public decimal(): StringDefinition {
    return new StringDefinition(FieldType.Decimal)
  }

  /**
   * Create a new bytes field.
   */
  public bytes(): BytesDefinition {
    return new BytesDefinition(FieldType.Bytes)
  }

  /**
   * Create a new bigint field.
   */
  public bigint(): BigIntDefinition {
    return new BigIntDefinition(FieldType.BigInt)
  }

  /**
   * Create a new relation field.
   *
   * @param ref - A model to be referred. Takes either a model or a closure resolving to a model.
   */
  public relation(ref: RelationRef): RelationDefinition {
    return new RelationDefinition(ref)
  }

  /**
   * Create a new reference field, referencing a type.
   *
   * @param type - A type to be referred.
   */
  public ref(type: Type | string): ReferenceDefinition {
    return new ReferenceDefinition(type)
  }

  /**
   * Create a new enum field.
   *
   * @param e - An enum to be referred.
   */
  public enumRef<T extends string, U extends EnumShape<T>>(
    e: Enum<T, U>
  ): EnumDefinition<T, U> {
    return new EnumDefinition(e)
  }

  /**
   * Create a new field from an input object reference.
   *
   * @param input - The input object reference.
   */
  public inputRef(input: Input): InputDefinition {
    return new InputDefinition(input)
  }

  /**
   * Extends an existing type with the given queries.
   *
   * @param type - Either a type if the given type is directly in the schema,
   *               or a string if extending an external type introspected from an
   *               external datasource.
   * @param definition - A collection of queries to be added to the extension.
   */
  public extend(type: string | Type, definition: Record<string, QueryInput>) {
    const extension = new TypeExtension(type)

    Object.entries(definition).forEach(([name, input]) => {
      const query = new Query(name, input.returns, input.resolver)

      if (input.args != null) {
        Object.entries(input.args).forEach(([name, type]) =>
          query.argument(name, type)
        )
      }

      extension.query(query)
    })

    this.extendedTypes.push(extension)
  }

  /**
   * Returns the environment variable with the given variableName.
   * Throws, if the variable is not set.
   *
   * @param variableName - The name of the environment variable.
   */
  public env(variableName: string): string {
    const value = process.env[variableName]

    if (value === undefined || value === null) {
      throw `Environment variable ${variableName} is not set`
    }

    return value
  }

  /**
   * Empty the schema.
   */
  public clear() {
    this.queries = undefined
    this.mutations = undefined
    this.interfaces = []
    this.types = []
    this.unions = []
    this.enums = []
    this.models = []
    this.datasources = new Datasources()
    this.extendedTypes = []
    this.inputs = []
  }

  public toString(): string {
    this.datasources.inner.forEach((datasource) => {
      if (datasource instanceof MongoDBAPI) {
        this.models = this.models.concat(datasource.models)
      }
    })

    const datasources = this.datasources.toString()
    const interfaces = this.interfaces.map(String).join('\n\n')
    const types = this.types.map(String).join('\n\n')
    const inputs = this.inputs.map(String).join('\n\n')
    const queries = this.queries ? this.queries.toString() : ''
    const mutations = this.mutations ? this.mutations.toString() : ''
    const extendedTypes = this.extendedTypes.map(String).join('\n\n')
    const unions = this.unions.map(String).join('\n\n')
    const enums = this.enums.map(String).join('\n\n')
    const models = this.models.map(String).join('\n\n')
    const extendSchema =
      datasources.length > 0 || this.federation ? 'extend schema ' : ''

    const renderOrder = [
      extendSchema,
      datasources,
      this.federation,
      interfaces,
      enums,
      inputs,
      types,
      extendedTypes,
      queries,
      mutations,
      unions,
      models
    ]

    return renderOrder.filter(Boolean).flat().map(String).join('\n\n')
  }
}

export class FederatedGraph {
  public toString(): string {
    return `\nextend schema @graph(type: federated)\n`
  }
}
