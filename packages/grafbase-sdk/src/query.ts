import { ListDefinition } from './field/list'
import { ReferenceDefinition } from './reference'
import { ScalarDefinition } from './typedefs/scalar'

export type InputType = ScalarDefinition | ListDefinition | ReferenceDefinition
export type OutputType = ScalarDefinition | ListDefinition | ReferenceDefinition

export interface QueryInput {
  args?: Record<string, InputType>
  returns: OutputType
  resolver: string
}

export class QueryArgument {
  name: string
  type: InputType

  constructor(name: string, type: InputType) {
    this.name = name
    this.type = type
  }

  public toString(): string {
    return `${this.name}: ${this.type}`
  }
}

export class Query {
  name: string
  arguments: QueryArgument[]
  returns: OutputType
  resolver: string

  constructor(name: string, returnType: OutputType, resolverName: string) {
    this.name = name
    this.arguments = []
    this.returns = returnType
    this.resolver = resolverName
  }

  public pushArgument(name: string, type: InputType): Query {
    this.arguments.push(new QueryArgument(name, type))

    return this
  }

  public toString(): string {
    const args = this.arguments.map(String).join(', ')
    const argsStr = args ? `(${args})` : ''

    return `  ${this.name}${argsStr}: ${this.returns} @resolver(name: "${this.resolver}")`
  }
}
