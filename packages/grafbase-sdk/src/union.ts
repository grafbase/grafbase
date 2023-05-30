import { Type } from './type'
import { validateIdentifier } from './validation'

/**
 * A builder to create a GraphQL union.
 */
export class Union {
  name: string
  types: string[]

  constructor(name: string) {
    validateIdentifier(name)

    this.name = name
    this.types = []
  }

  /**
   * Push a new type to the union definition.
   *
   * @param type - The included type.
   */
  public type(type: Type): Union {
    this.types.push(type.name)

    return this
  }

  public toString(): string {
    const types = this.types.join(' | ')

    return `union ${this.name} = ${types}`
  }
}
