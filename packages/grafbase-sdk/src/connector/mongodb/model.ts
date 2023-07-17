import { Field } from '../../field'
import { Model } from '../../model'
import { AuthDefinition } from '../../typedefs/auth'
import { CacheDefinition } from '../../typedefs/cache'
import { DefaultDefinition } from '../../typedefs/default'
import { EnumDefinition } from '../../typedefs/enum'
import { LengthLimitedStringDefinition } from '../../typedefs/length-limited-string'
import { ListDefinition } from '../../typedefs/list'
import { MapDefinition } from '../../typedefs/map'
import { ReferenceDefinition } from '../../typedefs/reference'
import { ResolverDefinition } from '../../typedefs/resolver'
import { ScalarDefinition } from '../../typedefs/scalar'
import { UniqueDefinition } from '../../typedefs/unique'

/**
 * A collection of fields in a model.
 */
export type ModelFields = Record<string, FieldShape>

/**
 * A combination of classes a field in a model can be.
 */
export type FieldShape =
  | ScalarDefinition
  | ListDefinition
  | ReferenceDefinition
  | UniqueDefinition
  | DefaultDefinition
  | LengthLimitedStringDefinition
  | AuthDefinition
  | ResolverDefinition
  | CacheDefinition
  | MapDefinition
  | EnumDefinition<any, any>

export class MongoDBModel extends Model {
  connector: string
  collectionName?: string

  constructor(name: string, connector: string) {
    super(name)
    this.connector = connector
  }

  /**
   * Push a field to the model definition.
   *
   * @param name - The name of the model.
   * @param definition - Fields to be included in the model.
   */
  public field(name: string, definition: FieldShape): this {
    this.fields.push(new Field(name, definition))

    return this
  }

  /**
   * Set the name of the collection for this model in the database.
   * If not set, the name of the model is used.
   *
   * @param name - The name of the collection.
   */
  public collection(name: string): this {
    this.collectionName = name
    return this
  }

  public toString(): string {
    const auth = this.authRules ? ` @auth(\n    rules: ${this.authRules})` : ''
    const cache = this.cacheDirective ? ` ${this.cacheDirective}` : ''
    const collection = this.collectionName ?? this.name

    const header = `type ${this.name} @model(connector: "${this.connector}", collection: "${collection}")${auth}${cache} {`

    const fields = this.fields.map((field) => `  ${field}`).join('\n')

    const footer = '}'

    return `${header}\n${fields}\n${footer}`
  }
}
