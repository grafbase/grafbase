import { Type } from "./type"

export class Union {
  name: string
  types: string[]

  constructor(name: string) {
    this.name = name
    this.types = []
  }

  public type(type: Type): Union {
    this.types.push(type.name)

    return this
  }

  public toString(): string {
    const types = this.types.join(" | ")

    return `union ${this.name} = ${types}`
  }
}