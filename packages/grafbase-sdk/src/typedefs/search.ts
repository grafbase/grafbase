import { ListDefinition } from '../field/list'
import { ScalarDefinition } from './scalar'
import { UniqueDefinition } from './unique'

/**
 * A list of field types that can hold a `@search` attribute.
 */
export type Searchable = ScalarDefinition | ListDefinition | UniqueDefinition

export class SearchDefinition {
  field: Searchable

  constructor(field: Searchable) {
    this.field = field
  }

  public toString(): string {
    return `${this.field} @search`
  }
}
