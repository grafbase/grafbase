import { Field } from '../../field'
import { Model } from '../../model'
import { RelationDefinition } from '../../relation'
import { AuthDefinition } from '../../typedefs/auth'
import { CacheDefinition } from '../../typedefs/cache'
import { DefaultDefinition } from '../../typedefs/default'
import { EnumDefinition } from '../../typedefs/enum'
import { LengthLimitedStringDefinition } from '../../typedefs/length-limited-string'
import { ListDefinition, RelationListDefinition } from '../../typedefs/list'
import { MapDefinition } from '../../typedefs/map'
import { ReferenceDefinition } from '../../typedefs/reference'
import { ResolverDefinition } from '../../typedefs/resolver'
import { ScalarDefinition } from '../../typedefs/scalar'
import { SearchDefinition } from '../../typedefs/search'
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
  | MapDefinition
  | EnumDefinition<any, any>

export class DynamoDBModel extends Model {
  private isSearch: boolean

  constructor(name: string) {
    super(name)

    this.isSearch = false
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
   * Make the model searchable.
   */
  public search(): Model {
    this.isSearch = true

    return this
  }

  public toString(): string {
    const search = this.isSearch ? ' @search' : ''
    const auth = this.authRules ? ` @auth(\n    rules: ${this.authRules})` : ''
    const cache = this.cacheDirective ? ` ${this.cacheDirective}` : ''
    const header = `type ${this.name} @model${search}${auth}${cache} {`

    const fields = this.fields.map((field) => `  ${field}`).join('\n')

    const footer = '}'

    return `${header}\n${fields}\n${footer}`
  }
}
