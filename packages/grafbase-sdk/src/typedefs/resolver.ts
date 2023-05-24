import { ReferenceDefinition } from "../reference"
import { DefaultDefinition } from "./default"
import { ScalarDefinition } from "./scalar"
import { UniqueDefinition } from "./unique"

/**
 * A list of field types that can hold a `@resolver` attribute.
 */
export type Resolvable = ScalarDefinition
  | UniqueDefinition
  | DefaultDefinition
  | ReferenceDefinition

export class ResolverDefinition {
  field: Resolvable
  resolver: string

  constructor(field: Resolvable, resolver: string) {
    this.field = field
    this.resolver = resolver
  }

  public toString(): string {
    return `${this.field} @resolver(name: "${this.resolver}")`
  }
}