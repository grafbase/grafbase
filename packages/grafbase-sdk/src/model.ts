import { AuthRuleF, AuthRules } from './auth'
import { Field } from './field'
import { ListDefinition, RelationListDefinition } from './typedefs/list'
import { ReferenceDefinition } from './typedefs/reference'
import { RelationDefinition } from './relation'
import { AuthDefinition } from './typedefs/auth'
import {
  CacheDefinition,
  TypeCacheParams,
  TypeLevelCache
} from './typedefs/cache'
import { DefaultDefinition } from './typedefs/default'
import { LengthLimitedStringDefinition } from './typedefs/length-limited-string'
import { ResolverDefinition } from './typedefs/resolver'
import { ScalarDefinition } from './typedefs/scalar'
import { SearchDefinition } from './typedefs/search'
import { UniqueDefinition } from './typedefs/unique'
import { EnumDefinition } from './typedefs/enum'
import { validateIdentifier } from './validation'

/**
 * A collection of fields in a model.
 */
export type ModelFields = Record<string, ModelFieldShape>

/**
 * A combination of classes a field in a model can be.
 */
export type ModelFieldShape =
  | ScalarDefinition
  | RelationDefinition
  | ListDefinition
  | RelationListDefinition
  | SearchDefinition
  | ReferenceDefinition
  | UniqueDefinition
  | DefaultDefinition
  | LengthLimitedStringDefinition
  | AuthDefinition
  | ResolverDefinition
  | CacheDefinition
  | EnumDefinition<any, any>

export class Model {
  private _name: string
  private fields: Field[]
  private authRules?: AuthRules
  private isSearch: boolean
  private isLive: boolean
  private cacheDirective?: TypeLevelCache

  constructor(name: string) {
    validateIdentifier(name)

    this._name = name
    this.fields = []
    this.isSearch = false
    this.isLive = false
  }

  /**
   * Get the name of the model.
   */
  public get name(): string {
    return this._name
  }

  /**
   * Push a field to the model definition.
   *
   * @param name - The name of the model.
   * @param definition - Fields to be included in the model.
   */
  public field(name: string, definition: ModelFieldShape): Model {
    this.fields.push(new Field(name, definition))

    return this
  }

  /**
   * Make the model searchable.
   */
  public search(): Model {
    this.isSearch = true

    return this
  }

  /**
   * Enable live queries to the model.
   */
  public live(): Model {
    this.isLive = true

    return this
  }

  /**
   * Set the per-model `@auth` directive.
   *
   * @param rules - A closure to build the authentication rules.
   */
  public auth(rules: AuthRuleF): Model {
    const authRules = new AuthRules()
    rules(authRules)
    this.authRules = authRules

    return this
  }

  /**
   * Set the per-model `@cache` directive.
   *
   * @param params - The cache definition parameters.
   */
  public cache(params: TypeCacheParams): Model {
    this.cacheDirective = new TypeLevelCache(params)

    return this
  }

  public toString(): string {
    const search = this.isSearch ? ' @search' : ''
    const live = this.isLive ? ' @live' : ''
    const auth = this.authRules ? ` @auth(\n    rules: ${this.authRules})` : ''
    const cache = this.cacheDirective ? ` ${this.cacheDirective}` : ''
    const header = `type ${this.name} @model${search}${live}${auth}${cache} {`

    const fields = this.fields.map((field) => `  ${field}`).join('\n')

    const footer = '}'

    return `${header}\n${fields}\n${footer}`
  }
}
