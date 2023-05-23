import { AuthRuleF } from '../auth'
import { ListDefinition } from '../field/list'
import { AuthDefinition } from './auth'
import { LengthLimitedStringDefinition } from './length-limited-string'
import { ScalarDefinition } from './scalar'
import { UniqueDefinition } from './unique'

/**
 * A list of field types that can hold a `@search` attribute.
 */
export type Searchable =
  | ScalarDefinition
  | ListDefinition
  | UniqueDefinition
  | LengthLimitedStringDefinition

export class SearchDefinition {
  field: Searchable

  constructor(field: Searchable) {
    this.field = field
  }

  public auth(rules: AuthRuleF): AuthDefinition {
    return new AuthDefinition(this, rules)
  }

  public toString(): string {
    return `${this.field} @search`
  }
}
