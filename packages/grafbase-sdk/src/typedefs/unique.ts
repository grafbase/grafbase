import { AuthRuleF } from '../auth'
import { AuthDefinition } from './auth'
import { DefaultDefinition } from './default'
import { LengthLimitedStringDefinition } from './length-limited-string'
import { ResolverDefinition } from './resolver'
import { ScalarDefinition } from './scalar'
import { SearchDefinition } from './search'

type UniqueScalarType =
  | ScalarDefinition
  | DefaultDefinition
  | SearchDefinition
  | LengthLimitedStringDefinition

export class UniqueDefinition {
  compoundScope?: string[]
  scalar: UniqueScalarType

  constructor(scalar: UniqueScalarType, scope?: string[]) {
    this.scalar = scalar
    this.compoundScope = scope
  }

  public search(): SearchDefinition {
    return new SearchDefinition(this)
  }

  public auth(rules: AuthRuleF): AuthDefinition {
    return new AuthDefinition(this, rules)
  }

  public resolver(name: string): ResolverDefinition {
    return new ResolverDefinition(this, name)
  }

  public toString(): string {
    const scope = this.compoundScope?.map((field) => `"${field}"`).join(', ')
    const scopeArray = scope ? `[${scope}]` : null

    return scopeArray
      ? `${this.scalar} @unique(fields: ${scopeArray})`
      : `${this.scalar} @unique`
  }
}
