import { RequireAtLeastOne } from 'type-fest'
import { Enum } from '../enum'
import {
  BooleanListDefinition,
  DateListDefinition,
  ListDefinition,
  NumberListDefinition,
  StringListDefinition
} from '../field/list'
import { FieldType } from '../field/typedefs'
import { DefaultDefinition } from './default'
import {
  FieldLength,
  LengthLimitedStringDefinition
} from './length-limited-string'
import { SearchDefinition } from './search'
import { UniqueDefinition } from './unique'

export type DefaultValueType = string | number | Date | object | boolean

export class ScalarDefinition {
  fieldType: FieldType | Enum
  isOptional: boolean
  defaultValue?: DefaultValueType

  constructor(fieldType: FieldType | Enum) {
    this.fieldType = fieldType
    this.isOptional = false
  }

  public optional(): ScalarDefinition {
    this.isOptional = true

    return this
  }

  public unique(scope?: string[]): UniqueDefinition {
    return new UniqueDefinition(this, scope)
  }

  public search(): SearchDefinition {
    return new SearchDefinition(this)
  }

  public list(): ListDefinition {
    return new ListDefinition(this)
  }

  fieldTypeVal(): FieldType | Enum {
    return this.fieldType
  }

  public toString(): string {
    const required = this.isOptional ? '' : '!'

    let fieldType
    if (this.fieldType instanceof Enum) {
      fieldType = this.fieldType.name
    } else {
      fieldType = this.fieldType.toString()
    }

    return `${fieldType}${required}`
  }
}

export class StringDefinition extends ScalarDefinition {
  public default(val: string): DefaultDefinition {
    return new DefaultDefinition(this, val)
  }

  public length(
    fieldLength: RequireAtLeastOne<FieldLength, 'min' | 'max'>
  ): LengthLimitedStringDefinition {
    return new LengthLimitedStringDefinition(this, fieldLength)
  }

  public list(): StringListDefinition {
    return new StringListDefinition(this)
  }
}

export class NumberDefinition extends ScalarDefinition {
  public default(val: number): DefaultDefinition {
    return new DefaultDefinition(this, val)
  }

  public list(): NumberListDefinition {
    return new NumberListDefinition(this)
  }
}

export class BooleanDefinition extends ScalarDefinition {
  public default(val: boolean): DefaultDefinition {
    return new DefaultDefinition(this, val)
  }

  public list(): BooleanListDefinition {
    return new BooleanListDefinition(this)
  }
}

export class DateDefinition extends ScalarDefinition {
  public default(val: Date): DefaultDefinition {
    return new DefaultDefinition(this, val)
  }

  public list(): DateListDefinition {
    return new DateListDefinition(this)
  }
}

export class ObjectDefinition extends ScalarDefinition {
  public default(val: object): DefaultDefinition {
    return new DefaultDefinition(this, val)
  }
}
