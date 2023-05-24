import { Field } from './field'
import { ListDefinition } from './field/list'
import { Interface } from './interface'
import { ReferenceDefinition } from './reference'
import { ScalarDefinition } from './typedefs/scalar'

/**
 * A collection of fields in a model.
 */
export type TypeFields = Record<string, TypeFieldShape>

/**
 * A combination of classes a field in a non-model type can be.
 */
export type TypeFieldShape =
  | ScalarDefinition
  | ListDefinition
  | ReferenceDefinition

/**
 * A composite type definition (e.g. not a model).
 */
export class Type {
  name: string
  fields: Field[]
  interfaces: Interface[]

  constructor(name: string) {
    this.name = name
    this.fields = []
    this.interfaces = []
  }

  /**
  * Pushes a field to the type definition.
  */
  public field(name: string, definition: TypeFieldShape): Type {
    this.fields.push(new Field(name, definition))

    return this
  }

  /**
  * Pushes an interface implemented by the type.
  */
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
