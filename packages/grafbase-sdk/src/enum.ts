import { validateIdentifier } from "./validation"

/**
 * Defines how an input enum can look like. Either an array of
 * strings with at least one item, or a TypeScript enum definition.
 */
export type EnumShape<T> = [T, ...Array<T>]

export class Enum<T extends string, U extends EnumShape<T>> {
  name: string
  variants: U

  constructor(name: string, variants: U) {
    validateIdentifier(name)
    variants.forEach((variant) => validateIdentifier(variant))

    this.name = name
    this.variants = variants
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
