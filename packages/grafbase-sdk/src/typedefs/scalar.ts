import { RequireAtLeastOne } from 'type-fest'
import { Enum } from '../enum'
import {
  BigIntListDefinition,
  BooleanListDefinition,
  BytesListDefinition,
  DateListDefinition,
  DecimalListDefinition,
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
import { UniqueDefinition } from './unique'
import { AuthDefinition } from './auth'
import { AuthRuleF } from '../auth'
import { ResolverDefinition } from './resolver'
import { CacheDefinition, FieldCacheParams, FieldLevelCache } from './cache'
import { MapDefinition } from './map'
import { JoinDefinition } from './join'
import { DeprecatedDefinition } from './deprecated'
import { InaccessibleDefinition } from './inaccessible'
import { ShareableDefinition } from './shareable'
import { OverrideDefinition } from './override'
import { ProvidesDefinition } from './provides'
import { TagDefinition } from './tag'

export class ScalarDefinition {
  private _fieldType: FieldType | Enum<any, any>
  private isOptional: boolean
  protected defaultValue?: DefaultValueType

  constructor(fieldType: FieldType | Enum<any, any>) {
    this._fieldType = fieldType
    this.isOptional = false
  }

  /**
   * The type of the field
   */
  public get fieldType(): FieldType | Enum<any, any> {
    return this._fieldType
  }

  /**
   * Make the field optional.
   */
  public optional(): this {
    this.isOptional = true

    return this
  }

  /**
   * Make the field unique.
   *
   * @param scope - Additional fields to be added to the constraint.
   */
  public unique(scope?: string[]): UniqueDefinition {
    return new UniqueDefinition(this, scope)
  }

  /**
   * Allow multiple scalars to be used as values for the field.
   */
  public list(): ListDefinition {
    return new ListDefinition(this)
  }

  /**
   * Set the field-level auth directive.
   *
   * @param rules - A closure to build the authentication rules.
   */
  public auth(rules: AuthRuleF): AuthDefinition {
    return new AuthDefinition(this, rules)
  }

  /**
   * Set the field-level deprecated directive.
   *
   * @param rules - A closure to build the authentication rules.
   */
  public deprecated(reason?: string): DeprecatedDefinition {
    return new DeprecatedDefinition(this, reason ?? null)
  }

  /**
   * Attach a resolver function to the field.
   *
   * @param name - The name of the resolver function file without the extension or directory.
   */
  public resolver(name: string): ResolverDefinition {
    return new ResolverDefinition(this, name)
  }

  /**
   * Attach a join function to the field.
   *
   * @param select - The field selection string to join onto this field
   */
  public join(select: string): JoinDefinition {
    return new JoinDefinition(this, select)
  }

  /**
   * Set the field-level cache directive.
   *
   * @param params - The cache definition parameters.
   */
  public cache(params: FieldCacheParams): CacheDefinition {
    return new CacheDefinition(this, new FieldLevelCache(params))
  }

  /**
   * Sets the name of the field in the database, if different than the name of the field.
   *
   * Only supported on MongoDB.
   */
  public mapped(name: string): MapDefinition {
    return new MapDefinition(this, name)
  }

  /**
   * Set the field-level inaccessible directive.
   */
  public inaccessible(): InaccessibleDefinition {
    return new InaccessibleDefinition(this)
  }

  /**
   * Set the field-level shareable directive.
   */
  public shareable(): ShareableDefinition {
    return new ShareableDefinition(this)
  }

  /**
   * Set the field-level override directive.
   */
  public override(from: string): OverrideDefinition {
    return new OverrideDefinition(this, from)
  }

  /**
   * Set the field-level provides directive.
   */
  public provides(fields: string): ProvidesDefinition {
    return new ProvidesDefinition(this, fields)
  }

  /**
   * Adds a tag to this field
   *
   * @param tag - The tag to add
   */
  public tag(tag: string): TagDefinition {
    return new TagDefinition(this, tag)
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

export class DecimalDefinition extends ScalarDefinition {
  /**
   * Set the default value of the field.
   *
   * @param value - The value written to the database.
   */
  public default(value: string): DefaultDefinition {
    return new DefaultDefinition(this, value)
  }

  /**
   * Allow multiple scalars to be used as values for the field.
   */
  public list(): DecimalListDefinition {
    return new DecimalListDefinition(this)
  }
}

export class BytesDefinition extends ScalarDefinition {
  /**
   * Set the default value of the field.
   *
   * @param value - The value written to the database.
   */
  public default(value: string): DefaultDefinition {
    return new DefaultDefinition(this, value)
  }

  /**
   * Allow multiple scalars to be used as values for the field.
   */
  public list(): BytesListDefinition {
    return new BytesListDefinition(this)
  }
}

export class BigIntDefinition extends ScalarDefinition {
  /**
   * Set the default value of the field.
   *
   * @param value - The value written to the database.
   */
  public default(value: string): DefaultDefinition {
    return new DefaultDefinition(this, value)
  }

  /**
   * Allow multiple scalars to be used as values for the field.
   */
  public list(): BigIntListDefinition {
    return new BigIntListDefinition(this)
  }
}

export class StringDefinition extends ScalarDefinition {
  /**
   * Set the default value of the field.
   *
   * @param value - The value written to the database.
   */
  public default(value: string): DefaultDefinition {
    return new DefaultDefinition(this, value)
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
   * Allow multiple scalars to be used as values for the field.
   */
  public list(): StringListDefinition {
    return new StringListDefinition(this)
  }
}

export class NumberDefinition extends ScalarDefinition {
  /**
   * Set the default value of the field.
   *
   * @param value - The value written to the database.
   */
  public default(value: number): DefaultDefinition {
    return new DefaultDefinition(this, value)
  }

  /**
   * Allow multiple scalars to be used as values for the field.
   */
  public list(): NumberListDefinition {
    return new NumberListDefinition(this)
  }
}

export class BooleanDefinition extends ScalarDefinition {
  /**
   * Set the default value of the field.
   *
   * @param value - The value written to the database.
   */
  public default(value: boolean): DefaultDefinition {
    return new DefaultDefinition(this, value)
  }

  /**
   * Allow multiple scalars to be used as values for the field.
   */
  public list(): BooleanListDefinition {
    return new BooleanListDefinition(this)
  }
}

export class DateDefinition extends ScalarDefinition {
  /**
   * Set the default value of the field.
   *
   * @param value - The value written to the database.
   */
  public default(value: Date): DefaultDefinition {
    return new DefaultDefinition(this, value)
  }

  /**
   * Allow multiple scalars to be used as values for the field.
   */
  public list(): DateListDefinition {
    return new DateListDefinition(this)
  }
}

export class ObjectDefinition extends ScalarDefinition {
  /**
   * Set the default value of the field.
   *
   * @param value - The value written to the database.
   */
  public default(value: object): DefaultDefinition {
    return new DefaultDefinition(this, value)
  }
}
