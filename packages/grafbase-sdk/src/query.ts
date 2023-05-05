import { GListDef } from "./field/list"
import { GScalarDef } from "./field/typedefs"
import { GReferenceDef } from "./reference"

export type InputType = GScalarDef | GListDef | GReferenceDef
export type OutputType = GScalarDef | GListDef | GReferenceDef

export interface QueryInput {
  args: Record<string, InputType>,
  returns: OutputType
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

export enum QueryType {
  Query = "Query",
  Mutation = "Mutation",
}

export class PartialQuery {
  name: string
  arguments: QueryArgument[]
  returns: OutputType
  type: QueryType

  constructor(name: string, type: QueryType, returns: OutputType) {
    this.name = name
    this.arguments = []
    this.returns = returns
    this.type = type
  }

  public argument(name: string, inputType: InputType): PartialQuery {
    this.arguments.push(new QueryArgument(name, inputType))

    return this
  }

  public resolver(resolverName: string): Query {
    return new Query(this.name, this.arguments, this.returns, resolverName, this.type)
  }
}

export class Query {
  name: string
  arguments: QueryArgument[]
  returns: OutputType
  resolver: string
  type: QueryType

  constructor(
    name: string,
    args: QueryArgument[],
    returnType: OutputType,
    resolverName: string,
    type: QueryType
  ) {
    this.name = name
    this.arguments = args
    this.returns = returnType
    this.resolver = resolverName
    this.type = type
  }

  public toString(): string {
    let header = `extend type ${this.type} {`
    let args = this.arguments.map(String).join(", ")
    let query = `  ${this.name}(${args}): ${this.returns} @resolver(name: "${this.resolver}")`
    let footer = "}"

    return `${header}\n${query}\n${footer}`
  }
}