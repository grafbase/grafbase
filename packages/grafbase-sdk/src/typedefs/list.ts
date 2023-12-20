import { AuthRuleF, AuthRules } from '../auth'
import { ReferenceDefinition } from './reference'
import { DefaultValueType, renderDefault } from './default'
import {
  BigIntDefinition,
  BooleanDefinition,
  BytesDefinition,
  DateDefinition,
  DecimalDefinition,
  NumberDefinition,
  ScalarDefinition,
  StringDefinition
} from './scalar'
import { SearchDefinition } from './search'
import { FieldType } from '../typedefs'
import { EnumDefinition } from './enum'
import { InputDefinition } from './input'
import { RequireAtLeastOne } from 'type-fest'
import { FieldLength } from './length-limited-string'
import { LengthLimitedStringDefinition } from './length-limited-string'
import { MapDefinition } from './map'
import { escapeString } from '../utils'

export type ListScalarType =
  | ScalarDefinition
  | ReferenceDefinition
  | EnumDefinition<any, any>
  | InputDefinition

export class ListDefinition {
  private fieldDefinition: ListScalarType
  private isOptional: boolean
  protected defaultValue?: DefaultValueType[]
  private authRules?: AuthRules
  private resolverName?: string
  private joinSelect?: string
  private otherDirectives: string[]

  constructor(fieldDefinition: ListScalarType) {
    this.fieldDefinition = fieldDefinition
    this.isOptional = false
    this.otherDirectives = []
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

  /**
   * Attach a join function to the field.
   *
   * @param select - The field selection string to join onto this field
   */
  public join(select: string): ListDefinition {
    this.joinSelect = select

    return this
  }

  /**
   * Sets the name of the field in the database, if different than the name of the field.
   *
   * @param name - The mapped name
   */
  public mapped(name: string): MapDefinition {
    return new MapDefinition(this, name)
  }

  /**
   * Adds a tag to this field
   *
   * @param tag - The tag to add
   */
  public tag(name: string): ListDefinition {
    this.otherDirectives.push(`@tag(name: ${escapeString(name)})`)
    return this
  }

  /**
   * Set the field-level inaccessible directive.
   */
  public inaccessible(): ListDefinition {
    this.otherDirectives.push(`@inaccessible`)
    return this
  }

  /**
   * Set the field-level shareable directive.
   */
  public shareable(): ListDefinition {
    this.otherDirectives.push(`@shareable`)
    return this
  }

  /**
   * Set the field-level override directive.
   */
  public override(from: string): ListDefinition {
    this.otherDirectives.push(`@override(from: ${from})`)
    return this
  }

  /**
   * Set the field-level provides directive.
   */
  public provides(fields: string): ListDefinition {
    this.otherDirectives.push(`@provides(fields: ${fields})`)
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

    const join = this.joinSelect ? ` @join(select: "${this.joinSelect}")` : ''

    const otherDirectives =
      this.otherDirectives.length != 0
        ? ` ${this.otherDirectives.join(' ')}`
        : ''

    return `[${this.fieldDefinition}]${required}${rules}${resolver}${join}${otherDirectives}`
  }
}

class ListWithDefaultDefinition extends ListDefinition {
  protected _fieldType: FieldType

  constructor(fieldDefinition: ScalarDefinition) {
    super(fieldDefinition)

    this._fieldType = fieldDefinition.fieldType as FieldType
  }

  public toString(): string {
    const defaultValue =
      this.defaultValue != null
        ? ` @default(value: [${this.defaultValue
            .map((v) => renderDefault(v, this._fieldType))
            .join(', ')}])`
        : ''

    return `${super.toString()}${defaultValue}`
  }
}

export class DecimalListDefinition extends ListWithDefaultDefinition {
  constructor(fieldDefinition: DecimalDefinition) {
    super(fieldDefinition)
  }

  /**
   * The type of the field
   */
  public get fieldType(): FieldType {
    return this._fieldType
  }

  /**
   * Set the default value of the field.
   *
   * @param value - The value written to the database.
   */
  public default(val: string[]): this {
    this.defaultValue = val

    return this
  }
}

export class BytesListDefinition extends ListWithDefaultDefinition {
  constructor(fieldDefinition: BytesDefinition) {
    super(fieldDefinition)
  }

  /**
   * The type of the field
   */
  public get fieldType(): FieldType {
    return this._fieldType
  }

  /**
   * Set the default value of the field.
   *
   * @param value - The value written to the database.
   */
  public default(val: string[]): this {
    this.defaultValue = val

    return this
  }
}

export class BigIntListDefinition extends ListWithDefaultDefinition {
  constructor(fieldDefinition: BigIntDefinition) {
    super(fieldDefinition)
  }

  /**
   * The type of the field
   */
  public get fieldType(): FieldType {
    return this._fieldType
  }

  /**
   * Set the default value of the field.
   *
   * @param value - The value written to the database.
   */
  public default(val: string[]): this {
    this.defaultValue = val

    return this
  }
}

export class StringListDefinition extends ListWithDefaultDefinition {
  constructor(fieldDefinition: StringDefinition) {
    super(fieldDefinition)
  }

  /**
   * The type of the field
   */
  public get fieldType(): FieldType {
    return this._fieldType
  }

  /**
   * Specify a minimum or a maximum (or both) length of the field.
   *
   * @param fieldLength - Either `min`, `max` or both.
   */
  public length(
    fieldLength: RequireAtLeastOne<FieldLength, 'min' | 'max'>
  ): LengthLimitedStringDefinition {
    return new LengthLimitedStringDefinition(this, fieldLength)
  }

  /**
   * Sets the name of the field in the database, if different than the name of the field.
   *
   * @param name - The mapped name
   */
  public map(name: string): MapDefinition {
    return new MapDefinition(this, name)
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
