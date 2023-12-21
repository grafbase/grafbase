import { EnumDefinition } from './typedefs/enum'
import { InputDefinition } from './typedefs/input'
import { ListDefinition } from './typedefs/list'
import { ScalarDefinition } from './typedefs/scalar'
import { validateIdentifier } from './validation'

export type InputFields = Record<string, InputFieldShape>

export type InputFieldShape =
  | ScalarDefinition
  | ListDefinition
  | EnumDefinition<any, any>
  | InputDefinition

/**
 * A GraphQL Input Object defines a set of input fields, used in queries and mutations.
 */
export class Input {
  private _name: string
  private _kind: 'input'
  private fields: InputField[]

  constructor(name: string) {
    validateIdentifier(name)

    this._name = name
    this._kind = 'input'
    this.fields = []
  }

  /**
   * The name of the input.
   */
  public get name(): string {
    return this._name
  }

  public get kind(): 'input' {
    return this._kind
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
