import { RequireAtLeastOne } from 'type-fest'
import { Enum } from '../enum'
import {
  BooleanListDefinition,
  DateListDefinition,
  ListDefinition,
  NumberListDefinition,
  StringListDefinition
} from './list'
import { FieldType } from '../typedefs'
import { DefaultDefinition, DefaultValueType } from './default'
import {
  FieldLength,
  LengthLimitedStringDefinition
} from './length-limited-string'
import { SearchDefinition } from './search'
import { UniqueDefinition } from './unique'
import { AuthDefinition } from './auth'
import { AuthRuleF } from '../auth'
import { ResolverDefinition } from './resolver'
import { CacheDefinition, CacheParams, TypeLevelCache } from './cache'

export class ScalarDefinition {
  fieldType: FieldType | Enum<any, any>
  isOptional: boolean
  defaultValue?: DefaultValueType

  constructor(fieldType: FieldType | Enum<any, any>) {
    this.fieldType = fieldType
    this.isOptional = false
  }

  public optional(): this {
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

  public auth(rules: AuthRuleF): AuthDefinition {
    return new AuthDefinition(this, rules)
  }

  public resolver(name: string): ResolverDefinition {
    return new ResolverDefinition(this, name)
  }

  public cache(params: CacheParams): CacheDefinition {
    return new CacheDefinition(this, new TypeLevelCache(params))
  }

  fieldTypeVal(): FieldType | Enum<any, any> {
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
