import { AtLeastOne } from '.'

/**
* Defines how an input enum can look like. Either an array of
* strings with at least one item, or a TypeScript enum definition.
*/
export type EnumShape = AtLeastOne<string> | { [s: number]: string }

export class Enum {
  name: string
  variants: string[]

  constructor(name: string, variants: EnumShape) {
    this.name = name

    if (Array.isArray(variants)) {
      this.variants = variants
    } else {
      this.variants = Object.keys(variants).filter((key) => isNaN(Number(key)))
    }
  }

  public toString(): string {
    const header = `enum ${this.name} {`
    const variants = this.variants.map((variant) => `  ${variant}`).join(',\n')
    const footer = '}'

    return `${header}\n${variants}\n${footer}`
  }
}
