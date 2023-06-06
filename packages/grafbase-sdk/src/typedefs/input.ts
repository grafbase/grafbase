import { Input } from "../input_type"
import { ListDefinition } from "./list"

/**
 * Defines a reference to an input object
 */
export class InputDefinition {
  name: string
  isOptional: boolean

  constructor(input: Input) {
    this.name = input.name
    this.isOptional = false
  }

  /**
   * Set the field optional.
   */
  public optional(): this {
    this.isOptional = true

    return this
  }

  /**
   * Allow multiple scalars to be used as values for the field.
   */
  public list(): ListDefinition {
    return new ListDefinition(this)
  }

  public toString(): string {
    const required = this.isOptional ? '' : '!'

    return `${this.name}${required}`
  }
}