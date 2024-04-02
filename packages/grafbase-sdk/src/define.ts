import { Enum, EnumShape } from './enum'
import { Input, InputFields } from './input_type'
import { Interface, InterfaceFields } from './interface'
import { Query, QueryInput } from './query'
import { Type, TypeFields } from './type'
import { EnumDefinition } from './typedefs/enum'
import { InputDefinition } from './typedefs/input'
import { ReferenceDefinition } from './typedefs/reference'
import { Union } from './union'

export default {
  /**
   * Creates a new object type.
   *
   * @param name - The name of the type.
   * @param fields - The fields to be included.
   */
  type: (name: string, fields: TypeFields) =>
    Object.entries(fields).reduce(
      (type, [name, definition]) => type.field(name, definition),
      new Type(name)
    ),

  /**
   * Creates a new interface.
   *
   * @param name - The name of the interface.
   * @param fields - The fields to be included.
   */
  interface: (name: string, fields: InterfaceFields) =>
    Object.entries(fields).reduce(
      (iface, [name, definition]) => iface.field(name, definition),
      new Interface(name)
    ),

  /**
   * Creates a new union.
   *
   * @param name - The name of the union.
   * @param types - The types to be included.
   */
  union: (name: string, types: Record<string, Type>) =>
    Object.entries(types).reduce(
      (model, [_, type]) => model.type(type),
      new Union(name)
    ),

  /**
   * Creates a new query
   *
   * @param name - The name of the query.
   * @param definition - The query definition.
   */
  query: (name: string, definition: QueryInput) => {
    const query = new Query(
      name,
      definition.returns,
      definition.resolver,
      false
    )

    if (definition.args != null) {
      Object.entries(definition.args).forEach(([name, type]) =>
        query.argument(name, type)
      )
    }

    return query
  },

  /**
   * Creates a new mutation.
   *
   * @param name - The name of the mutation.
   * @param fields - The mutation definition.
   */
  mutation: (name: string, definition: QueryInput) => {
    const mutation = new Query(
      name,
      definition.returns,
      definition.resolver,
      true
    )

    if (definition.args != null) {
      Object.entries(definition.args).forEach(
        ([name, type]) => mutation.argument(name, type),
        mutation
      )
    }

    return mutation
  },

  /**
   * Creates a new input.
   *
   * @param name = The name of the input.
   * @param fields = The input definition.
   */
  input: (name: string, definition: InputFields) => {
    const input = new Input(name)

    Object.entries(definition).forEach(([name, type]) => {
      input.field(name, type)
    })

    return input
  },

  /**
   * Creates a new enum.
   *
   * @param name - The name of the enum.
   * @param variants - A list of variants of the enum.
   */
  enum: <T extends string, U extends EnumShape<T>>(name: string, variants: U) =>
    new Enum(name, variants),

  /**
   * Create a new reference field, referencing a type.
   *
   * @param type - A type to be referred.
   */
  ref: (type: Type | Union | string) => new ReferenceDefinition(type),

  /**
   * Create a new enum field.
   *
   * @param definition - An enum to be referred.
   */
  enumRef: <T extends string, U extends EnumShape<T>>(definition: Enum<T, U>) =>
    new EnumDefinition(definition),

  /**
   * Create a new field from an input object reference.
   *
   * @param input - The input object reference.
   */
  inputRef: (input: Input) => new InputDefinition(input)
}
