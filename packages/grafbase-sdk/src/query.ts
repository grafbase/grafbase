import {
  DefaultDefinition,
  DefaultFieldShape,
  DefaultValueType,
  renderDefault
} from './typedefs/default'
import { EnumDefinition } from './typedefs/enum'
import { InputDefinition } from './typedefs/input'
import { ListDefinition } from './typedefs/list'
import { ReferenceDefinition } from './typedefs/reference'
import { ScalarDefinition } from './typedefs/scalar'
import { validateIdentifier } from './validation'

/** The possible types of an input parameters of a query. */
export type InputType =
  | ScalarDefinition
  | ListDefinition
  | InputDefinition
  | InputDefaultDefinition
  | EnumDefinition<any, any>
  | ReferenceDefinition

/** The possible types of an output parameters of a query. */
export type OutputType = ScalarDefinition | ListDefinition | ReferenceDefinition

/**
 * Defaults are rendered differently in input types, which we do in this specialization
 */
export class InputDefaultDefinition extends DefaultDefinition {
  constructor(scalar: DefaultFieldShape, defaultValue: DefaultValueType) {
    super(scalar, defaultValue)
  }

  public toString(): string {
    const defaultValue = renderDefault(
      this._defaultValue,
      this._scalar.fieldTypeVal()
    )
    return `${this._scalar} = ${defaultValue}`
  }
}

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
  private name: string
  private type: InputType

  constructor(name: string, type: InputType) {
    validateIdentifier(name)

    this.name = name

    if ('scalar' in type && 'defaultValue' in type) {
      this.type = new InputDefaultDefinition(type.scalar, type.defaultValue)
    } else {
      this.type = type
    }
  }

  public toString(): string {
    return `${this.name}: ${this.type}`
  }
}

/**
 * An edge resolver query definition.
 */
export class Query {
  private name: string
  private _kind: 'mutation' | 'query'
  private arguments: QueryArgument[]
  private returns: OutputType
  private resolver: string

  constructor(
    name: string,
    returnType: OutputType,
    resolverName: string,
    mutation: boolean
  ) {
    validateIdentifier(name)

    this.name = name
    this.arguments = []
    this.returns = returnType
    this.resolver = resolverName
    this._kind = mutation ? 'mutation' : 'query'
  }

  public get kind(): 'mutation' | 'query' {
    return this._kind
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

    return `${this.name}${argsStr}: ${this.returns} @resolver(name: "${this.resolver}")`
  }
}
