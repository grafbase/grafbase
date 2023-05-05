import { Field } from "./field"
import { GListDef } from "./field/list"
import { GScalarDef } from "./field/typedefs"
import { Interface } from "./interface"

export class Type {
  name: string
  fields: Field[]
  interface?: Interface

  constructor(name: string) {
    this.name = name
    this.fields = []
  }

  public field(name: string, definition: GScalarDef | GListDef): Type {
    this.fields.push(new Field(name, definition))

    return this
  }

  public implements(i: Interface): Type {
    this.interface = i

    return this
  }

  public toString(): string {
    const impl = this.interface ? ` implements ${this.interface?.name}` : ""
    const header = `type ${this.name}${impl} {`

    let fields = (this.interface?.fields ?? [])
      .concat(this.fields)
      .map((field) => `  ${field}`)
      .join("\n")

    const footer = '}'

    return `${header}\n${fields}\n${footer}`
  }
}
