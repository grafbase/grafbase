import { AuthRuleF } from '../auth'
import { ListDefinition } from './list'
import { Type } from '../type'
import { AuthDefinition } from './auth'
import { ResolverDefinition } from './resolver'

export class ReferenceDefinition {
  referencedType: string
  isOptional: boolean

  constructor(referencedType: Type) {
    this.referencedType = referencedType.name
    this.isOptional = false
  }

  public optional(): ReferenceDefinition {
    this.isOptional = true

    return this
  }

  public list(): ListDefinition {
    return new ListDefinition(this)
  }

  public auth(rules: AuthRuleF): AuthDefinition {
    return new AuthDefinition(this, rules)
  }

  public resolver(name: string): ResolverDefinition {
    return new ResolverDefinition(this, name)
  }

  public toString(): string {
    const required = this.isOptional ? '' : '!'

    return `${this.referencedType}${required}`
  }
}
