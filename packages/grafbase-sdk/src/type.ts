import { Field } from './field'
import { ListDefinition } from './typedefs/list'
import { Interface } from './interface'
import {
  CacheDefinition,
  TypeCacheParams,
  TypeLevelCache
} from './typedefs/cache'
import { ReferenceDefinition } from './typedefs/reference'
import { ScalarDefinition } from './typedefs/scalar'
import { EnumDefinition } from './typedefs/enum'
import { validateIdentifier } from './validation'

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
  | CacheDefinition
  | EnumDefinition<any, any>

/**
 * A composite type definition (e.g. not a model).
 */
export class Type {
  name: string
  fields: Field[]
  interfaces: Interface[]
  cacheDirective?: TypeLevelCache

  constructor(name: string) {
    validateIdentifier(name)

    this.name = name
    this.fields = []
    this.interfaces = []
  }

  /**
   * Pushes a field to the type definition.
   *
   * @param name - The name of the field.
   * @param definition - The type definition with optional attributes.
   */
  public field(name: string, definition: TypeFieldShape): this {
    this.fields.push(new Field(name, definition))

    return this
  }

  /**
   * Pushes an interface implemented by the type.
   *
   * @param iface - The interface this type implements.
   */
  public implements(iface: Interface): this {
    this.interfaces.push(iface)

    return this
  }

  /**
   * Sets the type `@cache` directive.
   *
   * @param params - The cache definition parameters.
   */
  public cache(params: TypeCacheParams): this {
    this.cacheDirective = new TypeLevelCache(params)

    return this
  }

  public toString(): string {
    const interfaces = this.interfaces.map((i) => i.name).join(' & ')
    const cache = this.cacheDirective ? ` ${this.cacheDirective}` : ''
    const impl = interfaces ? ` implements ${interfaces}` : ''
    const header = `type ${this.name}${cache}${impl} {`

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
