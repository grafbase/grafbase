import { ReferenceDefinition } from '../reference'
import { RelationDefinition } from '../relation'
import { renderDefault } from '../typedefs/default'
import {
  BooleanDefinition,
  DateDefinition,
  NumberDefinition,
  ScalarDefinition,
  DefaultValueType,
  StringDefinition
} from '../typedefs/scalar'
import { SearchDefinition } from '../typedefs/search'
import { FieldType } from './typedefs'

export class ListDefinition {
  fieldDefinition: ScalarDefinition | RelationDefinition | ReferenceDefinition
  isOptional: boolean
  defaultValue?: DefaultValueType[]

  constructor(
    fieldDefinition: ScalarDefinition | RelationDefinition | ReferenceDefinition
  ) {
    this.fieldDefinition = fieldDefinition
    this.isOptional = false
  }

  public optional(): ListDefinition {
    this.isOptional = true

    return this
  }

  public search(): SearchDefinition {
    return new SearchDefinition(this)
  }

  public toString(): string {
    const required = this.isOptional ? '' : '!'

    return `[${this.fieldDefinition}]${required}`
  }
}

export class RelationListDefinition {
  relation: RelationDefinition
  isOptional: boolean

  constructor(fieldDefinition: RelationDefinition) {
    this.relation = fieldDefinition
    this.isOptional = false
  }

  public optional(): RelationListDefinition {
    this.isOptional = true

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

    return `[${modelName}${relationRequired}]${listRequired}${relationAttribute}`
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

  public default(val: string[]): StringListDefinition {
    this.defaultValue = val

    return this
  }
}

export class NumberListDefinition extends ListWithDefaultDefinition {
  constructor(fieldDefinition: NumberDefinition) {
    super(fieldDefinition)
  }

  public default(val: number[]): NumberListDefinition {
    this.defaultValue = val

    return this
  }
}

export class BooleanListDefinition extends ListWithDefaultDefinition {
  constructor(fieldDefinition: BooleanDefinition) {
    super(fieldDefinition)
  }

  public default(val: boolean[]): BooleanListDefinition {
    this.defaultValue = val

    return this
  }
}

export class DateListDefinition extends ListWithDefaultDefinition {
  constructor(fieldDefinition: DateDefinition) {
    super(fieldDefinition)
  }

  public default(val: Date[]): DateListDefinition {
    this.defaultValue = val

    return this
  }
}
