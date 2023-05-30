import { ListDefinition } from './typedefs/list'
import { ReferenceDefinition } from './typedefs/reference'
import { ScalarDefinition } from './typedefs/scalar'
import { validateIdentifier } from './validation'

/** The possible types of an input parameters of a query. */
export type InputType = ScalarDefinition | ListDefinition | ReferenceDefinition

/** The possible types of an output parameters of a query. */
export type OutputType = ScalarDefinition | ListDefinition | ReferenceDefinition

/**
 * Parameters to create a new query definition.
 */
export interface QueryInput {
  args?: Record<string, InputType>
  returns: OutputType
  resolver: string
}

/**
 * An input argument shape of a query.
 */
export class QueryArgument {
  name: string
  type: InputType

  constructor(name: string, type: InputType) {
    validateIdentifier(name)

    this.name = name
    this.type = type
  }

  public toString(): string {
    return `${this.name}: ${this.type}`
  }
}

/**
 * An edge resolver query definition.
 */
export class Query {
  name: string
  arguments: QueryArgument[]
  returns: OutputType
  resolver: string

  constructor(name: string, returnType: OutputType, resolverName: string) {
    validateIdentifier(name)

    this.name = name
    this.arguments = []
    this.returns = returnType
    this.resolver = resolverName
  }

  /**
   * Push a new input argument to the query.
   *
   * @param name - The name of the input parameter.
   * @param type - The type of the input parameter.
   */
  public argument(name: string, type: InputType): Query {
    this.arguments.push(new QueryArgument(name, type))

    return this
  }

  public toString(): string {
    const args = this.arguments.map(String).join(', ')
    const argsStr = args ? `(${args})` : ''

    return `  ${this.name}${argsStr}: ${this.returns} @resolver(name: "${this.resolver}")`
  }
}
