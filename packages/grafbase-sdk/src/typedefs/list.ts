import { AuthRuleF, AuthRules } from '../auth'
import { ReferenceDefinition } from './reference'
import { RelationDefinition } from '../relation'
import { DefaultValueType, renderDefault } from './default'
import {
  BooleanDefinition,
  DateDefinition,
  NumberDefinition,
  ScalarDefinition,
  StringDefinition
} from './scalar'
import { SearchDefinition } from './search'
import { FieldType } from '../typedefs'
import { EnumDefinition } from './enum'
import { InputDefinition } from './input'

export type ListScalarType =
  | ScalarDefinition
  | RelationDefinition
  | ReferenceDefinition
  | EnumDefinition<any, any>
  | InputDefinition

export class ListDefinition {
  fieldDefinition: ListScalarType
  isOptional: boolean
  defaultValue?: DefaultValueType[]
  authRules?: AuthRules
  resolverName?: string

  constructor(fieldDefinition: ListScalarType) {
    this.fieldDefinition = fieldDefinition
    this.isOptional = false
  }

  /**
   * Make the field optional.
   */
  public optional(): ListDefinition {
    this.isOptional = true

    return this
  }

  /**
   * Make the field searchable.
   */
  public search(): SearchDefinition {
    return new SearchDefinition(this)
  }

  /**
   * Set the field-level auth directive.
   *
   * @param rules - A closure to build the authentication rules.
   */
  public auth(rules: AuthRuleF): ListDefinition {
    const authRules = new AuthRules()
    rules(authRules)

    this.authRules = authRules

    return this
  }

  /**
   * Attach a resolver function to the field.
   *
   * @param name - The name of the resolver function file without the extension or directory.
   */
  public resolver(name: string): ListDefinition {
    this.resolverName = name

    return this
  }

  public toString(): string {
    const required = this.isOptional ? '' : '!'

    const rules = this.authRules
      ? ` @auth(rules: ${this.authRules.toString().replace(/\s\s+/g, ' ')})`
      : ''

    const resolver = this.resolverName
      ? ` @resolver(name: "${this.resolverName}")`
      : ''

    return `[${this.fieldDefinition}]${required}${rules}${resolver}`
  }
}

export class RelationListDefinition {
  relation: RelationDefinition
  isOptional: boolean
  authRules?: AuthRules

  constructor(fieldDefinition: RelationDefinition) {
    this.relation = fieldDefinition
    this.isOptional = false
  }

  /**
   * Make the field optional.
   */
  public optional(): RelationListDefinition {
    this.isOptional = true

    return this
  }

  /**
   * Set the field-level auth directive.
   *
   * @param rules - A closure to build the authentication rules.
   */
  public auth(rules: AuthRuleF): RelationListDefinition {
    const authRules = new AuthRules()
    rules(authRules)

    this.authRules = authRules

    return this
  }

  public toString(): string {
    let modelName
    if (typeof this.relation.referencedModel === 'function') {
      modelName = this.relation.referencedModel().name
    } else {
      modelName = this.relation.referencedModel.name
    }

    const relationRequired = this.relation.isOptional ? '' : '!'
    const listRequired = this.isOptional ? '' : '!'

    const relationAttribute = this.relation.relationName
      ? ` @relation(name: ${this.relation.relationName})`
      : ''

    const rules = this.authRules
      ? ` @auth(rules: ${this.authRules.toString().replace(/\s\s+/g, ' ')})`
      : ''

    return `[${modelName}${relationRequired}]${listRequired}${relationAttribute}${rules}`
  }
}

class ListWithDefaultDefinition extends ListDefinition {
  fieldType: FieldType

  constructor(fieldDefinition: ScalarDefinition) {
    super(fieldDefinition)

    this.fieldType = fieldDefinition.fieldType as FieldType
  }

  public toString(): string {
    const defaultValue =
      this.defaultValue != null
        ? ` @default(value: [${this.defaultValue
            .map((v) => renderDefault(v, this.fieldType))
            .join(', ')}])`
        : ''

    return `${super.toString()}${defaultValue}`
  }
}

export class StringListDefinition extends ListWithDefaultDefinition {
  constructor(fieldDefinition: StringDefinition) {
    super(fieldDefinition)
  }

  /**
   * Set the default value of the field.
   *
   * @param value - The value written to the database.
   */
  public default(val: string[]): StringListDefinition {
    this.defaultValue = val

    return this
  }
}

export class NumberListDefinition extends ListWithDefaultDefinition {
  constructor(fieldDefinition: NumberDefinition) {
    super(fieldDefinition)
  }

  /**
   * Set the default value of the field.
   *
   * @param value - The value written to the database.
   */
  public default(val: number[]): NumberListDefinition {
    this.defaultValue = val

    return this
  }
}

export class BooleanListDefinition extends ListWithDefaultDefinition {
  constructor(fieldDefinition: BooleanDefinition) {
    super(fieldDefinition)
  }

  /**
   * Set the default value of the field.
   *
   * @param value - The value written to the database.
   */
  public default(val: boolean[]): BooleanListDefinition {
    this.defaultValue = val

    return this
  }
}

export class DateListDefinition extends ListWithDefaultDefinition {
  constructor(fieldDefinition: DateDefinition) {
    super(fieldDefinition)
  }

  /**
   * Set the default value of the field.
   *
   * @param value - The value written to the database.
   */
  public default(val: Date[]): DateListDefinition {
    this.defaultValue = val

    return this
  }
}
