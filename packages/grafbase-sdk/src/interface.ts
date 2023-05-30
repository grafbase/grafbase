import { Field } from './field'
import { ListDefinition } from './typedefs/list'
import { ScalarDefinition } from './typedefs/scalar'
import { validateIdentifier } from './validation'

/**
 * A collection of fields in an interface.
 */
export type InterfaceFields = Record<string, InterfaceFieldShape>

/**
 * A combination of classes a field in an interface can be.
 */
export type InterfaceFieldShape = ScalarDefinition | ListDefinition

export class Interface {
  name: string
  fields: Field[]

  constructor(name: string) {
    validateIdentifier(name)

    this.name = name
    this.fields = []
  }

  /**
   * Push a new field to the interface definition.
   *
   * @param name - The name of the field.
   * @param definition - The type and attirbutes of the field.
   */
  public field(name: string, definition: InterfaceFieldShape): Interface {
    this.fields.push(new Field(name, definition))

    return this
  }

  public toString(): string {
    const header = `interface ${this.name} {`
    const fields = this.fields.map((field) => `  ${field}`).join('\n')
    const footer = '}'

    return `${header}\n${fields}\n${footer}`
  }
}
