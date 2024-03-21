import { Input } from '../input_type'
import { InputType } from '../query'
import { ListDefinition } from './list'

/**
 * Defines a reference to an input object
 */
export class InputDefinition {
  private name: string
  private isOptional: boolean

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

  public get allArguments(): Record<string, InputType> {
    // Inputs never have arguments, but this type is valid inside ListScalarType
    // which needs to participate in the allArguments chain
    return {}
  }

  public toString(): string {
    const required = this.isOptional ? '' : '!'

    return `${this.name}${required}`
  }
}
