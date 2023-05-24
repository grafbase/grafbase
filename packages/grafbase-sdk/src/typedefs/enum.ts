import { AuthRuleF } from '../auth'
import { Enum, EnumShape } from '../enum'
import { AuthDefinition } from './auth'
import { DefaultDefinition } from './default'
import { ListDefinition } from './list'
import { SearchDefinition } from './search'
import { UniqueDefinition } from './unique'

export class EnumDefinition<T extends string, U extends EnumShape<T>> {
  enumName: string
  enumVariants: U
  isOptional: boolean

  constructor(referencedEnum: Enum<T, U>) {
    this.enumName = referencedEnum.name
    this.enumVariants = referencedEnum.variants
    this.isOptional = false
  }

  public optional(): this {
    this.isOptional = true

    return this
  }

  public list(): ListDefinition {
    return new ListDefinition(this)
  }

  public auth(rules: AuthRuleF): AuthDefinition {
    return new AuthDefinition(this, rules)
  }

  public search(): SearchDefinition {
    return new SearchDefinition(this)
  }

  public unique(scope?: string[]): UniqueDefinition {
    return new UniqueDefinition(this, scope)
  }

  public default(value: U[number]): DefaultDefinition {
    return new DefaultDefinition(this, value)
  }

  fieldTypeVal(): Enum<T, U> {
    return new Enum(this.enumName, this.enumVariants)
  }

  public toString(): string {
    const required = this.isOptional ? '' : '!'

    return `${this.enumName}${required}`
  }
}
