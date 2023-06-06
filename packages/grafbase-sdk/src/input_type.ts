import { EnumDefinition } from "./typedefs/enum";
import { InputDefinition } from "./typedefs/input";
import { ListDefinition } from "./typedefs/list";
import { ScalarDefinition } from "./typedefs/scalar";
import { validateIdentifier } from "./validation";

export type InputFields = Record<string, InputFieldShape>

export type InputFieldShape = ScalarDefinition
  | ListDefinition
  | EnumDefinition<any, any>
  | InputDefinition

/**
 * A GraphQL Input Object defines a set of input fields, used in queries and mutations.
 */
export class Input {
  public name: string
  private fields: InputField[]

  constructor(name: string) {
    validateIdentifier(name)

    this.name = name
    this.fields = []
  }
 
  /**
   * Pushes a field to the input definition.
   *
   * @param name - The name of the field.
   * @param definition - The type definition.
   */
  public field(name: string, definition: InputFieldShape): this {
    this.fields.push(new InputField(name, definition))

    return this
  }

  public toString(): string {
    const header = `input ${this.name} {`
    const fields = this.fields.map((f) => `  ${f}`).join('\n')
    const footer = '}'

    return `${header}\n${fields}\n${footer}`
  }
}

class InputField {
  private name: string
  private shape: InputFieldShape

  constructor(name: string, shape: InputFieldShape) {
    validateIdentifier(name)

    this.name = name
    this.shape = shape
  }

  public toString(): string {
    return `${this.name}: ${this.shape}`
  }
}