import { Field } from './field'
import { ListDefinition } from './typedefs/list'
import { ScalarDefinition } from './typedefs/scalar'

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
    this.name = name
    this.fields = []
  }

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
