import { FieldShape } from '.'
import { Field } from './field'

export class Model {
  name: string
  fields: Field[]
  isSearch: boolean
  isLive: boolean

  constructor(name: string) {
    this.name = name
    this.fields = []
    this.isSearch = false
    this.isLive = false
  }

  public field(name: string, definition: FieldShape): Model {
    this.fields.push(new Field(name, definition))

    return this
  }

  public search(): Model {
    this.isSearch = true;

    return this
  }

  public live(): Model {
    this.isLive = true;

    return this
  }

  public toString(): string {
    const search = this.isSearch ? " @search" : ""
    const live = this.isLive ? " @live" : ""
    const header = `type ${this.name} @model${search}${live} {`

    const fields = this
      .fields
      .map((field) => `  ${field}`).join("\n")

    const footer = '}'

    return `${header}\n${fields}\n${footer}`
  }
}
