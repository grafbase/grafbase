import { Field } from './field'
import { ListDefinition } from './field/list'
import { ScalarDefinition } from './field/typedefs'

export class Interface {
  name: string
  fields: Field[]

  constructor(name: string) {
    this.name = name
    this.fields = []
  }

  public field(
    name: string,
    definition: ScalarDefinition | ListDefinition
  ): Interface {
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
