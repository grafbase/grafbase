import { Model } from './model'
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
import { PostgresAPI, PartialPostgresAPI } from './connector/postgres'
import { FederatedGraphHeaders } from './federated/headers'
import { CacheParams, GlobalCache } from './cache'
import scalar from './scalar'
import create from './create'

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
  namespace?: boolean
}

const FEDERATION_VERSION = '2.3'

export class Graph {
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
  private subgraph: boolean

  constructor(subgraph: boolean) {
    this.enums = []
    this.types = []
    this.unions = []
    this.models = []
    this.interfaces = []
    this.datasources = new Datasources()
    this.extendedTypes = []
    this.inputs = []
    this.subgraph = subgraph
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
   * Add a new composite type to the schema.
   *
   * @param name - The name of the type.
   * @param fields - The fields to be included.
   */
  public type(name: string, fields: TypeFields): Type {
    const type = create.type(name, fields)

    this.addType(type)

    return type
  }

  /**
   * Add an existing type to the schema.
   *
   * @param type - The type to add
   */
  public addType(type: Type) {
    this.types.push(type)
  }

  /**
   * Add a new interface to the schema.
   *
   * @param name - The name of the interface.
   * @param fields - The fields to be included.
   */
  public interface(name: string, fields: InterfaceFields): Interface {
    const iface = create.interface(name, fields)

    this.addInterface(iface)

    return iface
  }

  /**
   * Add an existing interface to the schema.
   *
   * @param iface - The interface to add
   */
  public addInterface(iface: Interface) {
    this.interfaces.push(iface)
  }

  /**
   * Add a new union to the schema.
   *
   * @param name - The name of the union.
   * @param types - The types to be included.
   */
  public union(name: string, types: Record<string, Type>): Union {
    const union = create.union(name, types)

    this.addUnion(union)

    return union
  }

  /**
   * Add an existing union to the schema.
   *
   * @param union - The union to add
   */
  public addUnion(union: Union) {
    this.unions.push(union)
  }

  /**
   * Add a new query to the schema.
   *
   * @param name - The name of the query.
   * @param definition - The query definition.
   */
  public query(name: string, definition: QueryInput): Query {
    const query = create.query(name, definition)

    this.addQuery(query)

    return query
  }

  /**
   * Add an existing query to the schema.
   *
   * @param query - The query to add
   */
  public addQuery(query: Query) {
    if (!this.queries) {
      this.queries = new TypeExtension('Query')
    }

    this.queries.query(query)
  }

  /**
   * Add a new mutation to the schema.
   *
   * @param name - The name of the mutation.
   * @param fields - The mutation definition.
   */
  public mutation(name: string, definition: QueryInput): Query {
    const mutation = create.mutation(name, definition)

    this.addMutation(mutation)

    return mutation
  }

  /**
   * Add an existing mutation to the schema.
   *
   * @param mutation - The mutation to add
   */
  public addMutation(mutation: Query) {
    if (!this.mutations) {
      this.mutations = new TypeExtension('Mutation')
    }

    this.mutations.query(mutation)
  }

  /**
   * Add a new input to the schema.
   *
   * @param name = The name of the input.
   * @param fields = The input definition.
   */
  public input(name: string, definition: InputFields): Input {
    const input = create.input(name, definition)

    this.addInput(input)

    return input
  }

  /**
   * Add an existing input to the schema.
   *
   * @param input - The input to add
   */
  public addInput(input: Input) {
    this.inputs.push(input)
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
    const definition = create.enum(name, variants)

    this.addEnum(definition)

    return definition
  }

  /**
   * Add an existing enum to the schema.
   *
   * @param definition - The enum to add
   */
  public addEnum<T extends string, U extends EnumShape<T>>(
    definition: Enum<T, U>
  ) {
    this.enums.push(definition)
  }

  /**
   * Create a new string field.
   */
  public string(): StringDefinition {
    return scalar.string()
  }

  /**
   * Create a new ID field.
   */
  public id(): StringDefinition {
    return scalar.id()
  }

  /**
   * Create a new email field.
   */
  public email(): StringDefinition {
    return scalar.email()
  }

  /**
   * Create a new int field.
   */
  public int(): NumberDefinition {
    return scalar.int()
  }

  /**
   * Create a new float field.
   */
  public float(): NumberDefinition {
    return scalar.float()
  }

  /**
   * Create a new boolean field.
   */
  public boolean(): BooleanDefinition {
    return scalar.boolean()
  }

  /**
   * Create a new date field.
   */
  public date(): DateDefinition {
    return scalar.date()
  }

  /**
   * Create a new datetime field.
   */
  public datetime(): DateDefinition {
    return scalar.datetime()
  }

  /**
   * Create a new IP address field.
   */
  public ipAddress(): StringDefinition {
    return scalar.ipAddress()
  }

  /**
   * Create a new timestamp field.
   */
  public timestamp(): NumberDefinition {
    return scalar.timestamp()
  }

  /**
   * Create a new URL field.
   */
  public url(): StringDefinition {
    return scalar.url()
  }

  /**
   * Create a new JSON field.
   */
  public json(): ObjectDefinition {
    return scalar.json()
  }

  /**
   * Create a new phone number field.
   */
  public phoneNumber(): StringDefinition {
    return scalar.phoneNumber()
  }

  /**
   * Create a new decimal field.
   */
  public decimal(): StringDefinition {
    return scalar.decimal()
  }

  /**
   * Create a new bytes field.
   */
  public bytes(): BytesDefinition {
    return scalar.bytes()
  }

  /**
   * Create a new bigint field.
   */
  public bigint(): BigIntDefinition {
    return scalar.bigint()
  }

  /**
   * Create a new reference field, referencing a type.
   *
   * @param type - A type to be referred.
   */
  public ref(type: Type | string): ReferenceDefinition {
    return create.ref(type)
  }

  /**
   * Create a new enum field.
   *
   * @param e - An enum to be referred.
   */
  public enumRef<T extends string, U extends EnumShape<T>>(
    e: Enum<T, U>
  ): EnumDefinition<T, U> {
    return create.enumRef(e)
  }

  /**
   * Create a new field from an input object reference.
   *
   * @param input - The input object reference.
   */
  public inputRef(input: Input): InputDefinition {
    return create.inputRef(input)
  }

  /**
   * Extends an existing type with the given queries.
   *
   * @param type - Either a type if the given type is directly in the schema,
   *               or a string if extending an external type introspected from an
   *               external datasource.
   * @param definition - A collection of fields to be added to the extension
   *                     or a builder function if extending with directives
   */
  public extend(
    type: string | Type,
    definitionOrBuilder: Record<string, QueryInput> | DirectiveExtendFn
  ) {
    const extension = new TypeExtension(type)

    if (typeof definitionOrBuilder === 'function') {
      definitionOrBuilder(extension)
    } else {
      Object.entries(definitionOrBuilder).forEach(([name, input]) => {
        const query = new Query(name, input.returns, input.resolver)

        if (input.args != null) {
          Object.entries(input.args).forEach(([name, type]) =>
            query.argument(name, type)
          )
        }

        extension.query(query)
      })
    }

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

    const subgraph = this.subgraph
      ? `extend schema @federation(version: "${FEDERATION_VERSION}")`
      : ''
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

    const renderOrder = [
      subgraph,
      datasources,
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

export interface FederatedGraphInput {
  headers?: (headers: FederatedGraphHeaders) => void
  cache?: CacheParams
}

export class FederatedGraph {
  private readonly headers: FederatedGraphHeaders
  private readonly cache?: GlobalCache

  public constructor(input?: FederatedGraphInput) {
    this.headers = new FederatedGraphHeaders()
    if (input?.headers) {
      input.headers(this.headers)
    }

    if (input?.cache) {
      this.cache = new GlobalCache({ rules: input.cache.rules })
    }
  }

  public toString(): string {
    return `\nextend schema\n  @graph(type: federated)${this.headers}\n${
      this.cache || ''
    }`
  }
}

export type DirectiveExtendFn = (extend: TypeExtension) => void
