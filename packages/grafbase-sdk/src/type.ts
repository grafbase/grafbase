import { Field } from './field'
import { GListDef } from './field/list'
import { GScalarDef } from './field/typedefs'
import { Interface } from './interface'

export class Type {
  name: string
  fields: Field[]
  interfaces: Interface[]

  constructor(name: string) {
    this.name = name
    this.fields = []
    this.interfaces = []
  }

  public field(name: string, definition: GScalarDef | GListDef): Type {
    this.fields.push(new Field(name, definition))

    return this
  }

  public implements(i: Interface): Type {
    this.interfaces.push(i)

    return this
  }

  public toString(): string {
    const interfaces = this.interfaces.map((i) => i.name).join(' & ')
    const impl = interfaces ? ` implements ${interfaces}` : ''
    const header = `type ${this.name}${impl} {`

    const fields = distinct(
      (this.interfaces.flatMap((i) => i.fields) ?? []).concat(this.fields)
    )
      .map((field) => `  ${field}`)
      .join('\n')

    const footer = '}'

    return `${header}\n${fields}\n${footer}`
  }
}

function distinct(fields: Field[]): Field[] {
  const found = new Set()

  return fields.filter((f) => {
    if (found.has(f.name)) {
      return false
    } else {
      found.add(f.name)
      return true
    }
  })
}
