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
  private _name: string
  private _fields: Field[]
  private _kind: 'interface'

  constructor(name: string) {
    validateIdentifier(name)

    this._name = name
    this._fields = []
    this._kind = 'interface'
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

  /**
   * All fields that belong to the interface.
   */
  public get fields(): Field[] {
    return this._fields
  }

  public get kind(): 'interface' {
    return this._kind
  }

  /**
   * The name of the interface.
   */
  public get name(): string {
    return this._name
  }

  public toString(): string {
    const header = `interface ${this.name} {`
    const fields = this.fields.map((field) => `  ${field}`).join('\n')
    const footer = '}'

    return `${header}\n${fields}\n${footer}`
  }
}
