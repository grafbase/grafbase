import { Type } from './type'
import { validateIdentifier } from './validation'

/**
 * A builder to create a GraphQL union.
 */
export class Union {
  private name: string
  private _kind: 'union'
  private types: string[]

  constructor(name: string) {
    validateIdentifier(name)

    this.name = name
    this.types = []
    this._kind = 'union'
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

  public get kind(): 'union' {
    return this._kind
  }

  public toString(): string {
    const types = this.types.join(' | ')

    return `union ${this.name} = ${types}`
  }
}
