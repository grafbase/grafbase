import { Field } from './field'
import { ListDefinition } from './typedefs/list'
import { Interface } from './interface'
import {
  CacheDefinition,
  TypeCacheParams,
  TypeLevelCache
} from './typedefs/cache'
import { ReferenceDefinition } from './typedefs/reference'
import { ScalarDefinition } from './typedefs/scalar'
import { EnumDefinition } from './typedefs/enum'
import { ResolverDefinition } from './typedefs/resolver'
import { validateIdentifier } from './validation'
import { Query } from './query'
import { MapDefinition } from './typedefs/map'
import { FederationKey, FederationKeyParameters } from './federation'
import { JoinDefinition } from './typedefs/join'
import { TagDefinition } from './typedefs/tag'
import { InaccessibleDefinition } from './typedefs/inaccessible'
import { ShareableDefinition } from './typedefs/shareable'
import { OverrideDefinition } from './typedefs/override'
import { ProvidesDefinition } from './typedefs/provides'
import { DeprecatedDefinition } from './typedefs/deprecated'

/**
 * A collection of fields in a model.
 */
export type TypeFields = Record<string, TypeFieldShape>

/**
 * A combination of classes a field in a non-model type can be.
 */
export type TypeFieldShape =
  | ScalarDefinition
  | ListDefinition
  | ReferenceDefinition
  | CacheDefinition
  | MapDefinition
  | EnumDefinition<any, any>
  | ResolverDefinition
  | JoinDefinition
  | TagDefinition
  | InaccessibleDefinition
  | ShareableDefinition
  | OverrideDefinition
  | ProvidesDefinition
  | DeprecatedDefinition

/**
 * A composite type definition (e.g. not a model).
 */
export class Type {
  private _name: string
  private _kind: 'type'
  private fields: Field[]
  private interfaces: Interface[]
  private cacheDirective?: TypeLevelCache
  private keys: FederationKey[]

  constructor(name: string) {
    validateIdentifier(name)

    this._name = name
    this.fields = []
    this.interfaces = []
    this.keys = []
    this._kind = 'type'
  }

  /**
   * The name of the type.
   */
  public get name(): string {
    return this._name
  }

  public get kind(): 'type' {
    return this._kind
  }

  /**
   * Pushes a field to the type definition.
   *
   * @param name - The name of the field.
   * @param definition - The type definition with optional attributes.
   */
  public field(name: string, definition: TypeFieldShape): this {
    this.fields.push(new Field(name, definition))

    return this
  }

  /**
   * Pushes an interface implemented by the type.
   *
   * @param iface - The interface this type implements.
   */
  public implements(iface: Interface): this {
    this.interfaces.push(iface)

    return this
  }

  /**
   * Sets the type `@cache` directive.
   *
   * @param params - The cache definition parameters.
   */
  public cache(params: TypeCacheParams): this {
    this.cacheDirective = new TypeLevelCache(params)

    return this
  }

  /**
   * Marks this type as a federation entitiy with the given key
   *
   * @param fields The fields that make up this key, in FieldSet format
   * @param params The parameters for this key
   */
  public key(fields: string, params?: FederationKeyParameters): this {
    this.keys.push(new FederationKey(fields, params))
    return this
  }

  public toString(): string {
    const interfaces = this.interfaces.map((i) => i.name).join(' & ')
    const cache = this.cacheDirective ? ` ${this.cacheDirective}` : ''
    const keys =
      this.keys.length != 0
        ? ` ${this.keys.map((key) => key.toString()).join(' ')}`
        : ''
    const impl = interfaces ? ` implements ${interfaces}` : ''
    const header = `type ${this.name}${cache}${keys}${impl} {`

    const fields = distinct(
      (this.interfaces.flatMap((i) => i.fields) ?? []).concat(this.fields)
    )
      .map((field) => `  ${field}`)
      .join('\n')

    const footer = '}'

    return `${header}\n${fields}\n${footer}`
  }
}

export class TypeExtension {
  private name: string
  private queries: Query[]
  private keys: FederationKey[]
  private fieldExtensions: FieldExtension[]

  constructor(type: string | Type) {
    if (type instanceof Type) {
      this.name = type.name
    } else {
      validateIdentifier(type)
      this.name = type
    }

    this.queries = []
    this.keys = []
    this.fieldExtensions = []
  }

  /**
   * Pushes a query to the extension.
   *
   * @param query - The query to be added.
   */
  public query(query: Query): this {
    this.queries.push(query)

    return this
  }

  /**
   * Extends this type as a federation entity with the given key
   *
   * @param fields The fields that make up this key, in FieldSet format
   * @param params The parameters for this key
   */
  public key(fields: string, params?: FederationKeyParameters): this {
    this.keys.push(new FederationKey(fields, params))
    return this
  }

  /**
   * Extends a field of this type with additional federation directives
   *
   * @param field The name of the field to extend
   */
  public extendField(field: string): FieldExtension {
    const fieldExtension = new FieldExtension(field)
    this.fieldExtensions.push(fieldExtension)
    return fieldExtension
  }

  public toString(): string {
    const queries =
      this.queries.length > 0
        ? `{\n${this.queries.map(String).join('\n')}\n}`
        : ''

    const keys =
      this.keys.length > 0 ? this.keys.map((key) => ` \n  ${key}`) : ''

    const fieldExtends =
      this.fieldExtensions.length > 0
        ? this.fieldExtensions.map((field) => `\n  ${field}`)
        : ''

    return `extend type ${this.name} ${keys}${fieldExtends}${queries}`
  }
}

export class FieldExtension {
  private name: string
  private directives: string[]

  constructor(name: string) {
    this.name = name
    this.directives = []
  }

  /**
   * Adds an inaccessible directive to the field.
   */
  public inaccessible(): this {
    this.directives.push(`inaccesible: true`)
    return this
  }

  /**
   * Adds a shareable directive to the field.
   */
  public shareable(): this {
    this.directives.push(`shareable: true`)
    return this
  }

  /**
   * Adds a override directive to the field.
   */
  public override(from: string): this {
    this.directives.push(`override: {from: "${from}"}`)
    return this
  }

  /**
   * Adds a provides directive to the field.
   */
  public provides(fields: string): this {
    this.directives.push(`provides: {fields: "${fields}"}`)
    return this
  }

  public toString(): string {
    const directives = this.directives
      .map((directive) => `${directive}`)
      .join(', ')

    return `  @extendField(name: "${this.name}", ${directives})`
  }
}

function distinct(fields: Field[]): Field[] {
  const found = new Set()

  return fields.filter((f) => {
    if (found.has(f.name)) {
      return false
    } else {
      found.add(f.name)
      return true
    }
  })
}
