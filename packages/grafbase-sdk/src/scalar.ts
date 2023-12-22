import { FieldType } from './typedefs'
import {
  BigIntDefinition,
  BooleanDefinition,
  BytesDefinition,
  DateDefinition,
  NumberDefinition,
  ObjectDefinition,
  StringDefinition
} from './typedefs/scalar'

export default {
  /**
   * Create a new string field.
   */
  string: () => new StringDefinition(FieldType.String),

  /**
   * Create a new ID field.
   */
  id: () => new StringDefinition(FieldType.ID),

  /**
   * Create a new email field.
   */
  email: () => new StringDefinition(FieldType.Email),

  /**
   * Create a new int field.
   */
  int: () => new NumberDefinition(FieldType.Int),

  /**
   * Create a new float field.
   */
  float: () => new NumberDefinition(FieldType.Float),

  /**
   * Create a new boolean field.
   */
  boolean: () => new BooleanDefinition(FieldType.Boolean),

  /**
   * Create a new date field.
   */
  date: () => new DateDefinition(FieldType.Date),

  /**
   * Create a new datetime field.
   */
  datetime: () => new DateDefinition(FieldType.DateTime),

  /**
   * Create a new IP address field.
   */
  ipAddress: () => new StringDefinition(FieldType.IPAddress),

  /**
   * Create a new timestamp field.
   */
  timestamp: () => new NumberDefinition(FieldType.Timestamp),

  /**
   * Create a new URL field.
   */
  url: () => new StringDefinition(FieldType.URL),

  /**
   * Create a new JSON field.
   */
  json: () => new ObjectDefinition(FieldType.JSON),

  /**
   * Create a new phone number field.
   */
  phoneNumber: () => new StringDefinition(FieldType.PhoneNumber),

  /**
   * Create a new decimal field.
   */
  decimal: () => new StringDefinition(FieldType.Decimal),

  /**
   * Create a new bytes field.
   */
  bytes: () => new BytesDefinition(FieldType.Bytes),

  /**
   * Create a new bigint field.
   */
  bigint: () => new BigIntDefinition(FieldType.BigInt)
}
