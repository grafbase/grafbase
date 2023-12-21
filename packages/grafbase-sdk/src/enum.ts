import { validateIdentifier } from './validation'

/**
 * Defines how an input enum can look like. Either an array of
 * strings with at least one item, or a TypeScript enum definition.
 */
export type EnumShape<T> = [T, ...Array<T>]

export class Enum<T extends string, U extends EnumShape<T>> {
  private _name: string
  private _variants: U
  private _kind: 'enum'

  constructor(name: string, variants: U) {
    validateIdentifier(name)
    variants.forEach((variant) => validateIdentifier(variant))

    this._name = name
    this._variants = variants
    this._kind = 'enum'
  }

  /**
   * The name of the enum.
   */
  public get name(): string {
    return this._name
  }

  /**
   * A list of variants in the enum.
   */
  public get variants(): U {
    return this._variants
  }

  public get kind(): 'enum' {
    return this._kind
  }

  public toString(): string {
    const header = `enum ${this.name} {`
    const variants = this.variants
      .map((variant) => `  ${variant.toString()}`)
      .join(',\n')
    const footer = '}'

    return `${header}\n${variants}\n${footer}`
  }
}
