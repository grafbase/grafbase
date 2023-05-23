import { AuthRuleF, AuthRules } from './auth'
import { Field } from './field'
import { ListDefinition, RelationListDefinition } from './field/list'
import { ReferenceDefinition } from './reference'
import { RelationDefinition } from './relation'
import { DefaultDefinition } from './typedefs/default'
import { LengthLimitedStringDefinition } from './typedefs/length-limited-string'
import { ScalarDefinition } from './typedefs/scalar'
import { SearchDefinition } from './typedefs/search'
import { UniqueDefinition } from './typedefs/unique'

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

export class Model {
  name: string
  fields: Field[]
  authRules?: AuthRules
  isSearch: boolean
  isLive: boolean

  constructor(name: string) {
    this.name = name
    this.fields = []
    this.isSearch = false
    this.isLive = false
  }

  public field(name: string, definition: ModelFieldShape): Model {
    this.fields.push(new Field(name, definition))

    return this
  }

  public search(): Model {
    this.isSearch = true

    return this
  }

  public live(): Model {
    this.isLive = true

    return this
  }

  public auth(rules: AuthRuleF): Model {
    const authRules = new AuthRules()
    rules(authRules)
    this.authRules = authRules

    return this
  }

  public toString(): string {
    const search = this.isSearch ? ' @search' : ''
    const live = this.isLive ? ' @live' : ''
    const auth = this.authRules ? ` @auth(\n    rules: ${this.authRules})` : ''
    const header = `type ${this.name} @model${search}${live}${auth} {`

    const fields = this.fields.map((field) => `  ${field}`).join('\n')

    const footer = '}'

    return `${header}\n${fields}\n${footer}`
  }
}
