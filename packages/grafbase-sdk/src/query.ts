import { GListDef } from "./field/list"
import { GScalarDef } from "./field/typedefs"
import { GReferenceDef } from "./reference"

export type InputType = GScalarDef | GListDef | GReferenceDef
export type OutputType = GScalarDef | GListDef | GReferenceDef

export interface QueryInput {
  args: Record<string, InputType>,
  returns: OutputType,
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

export enum QueryType {
  Query = "Query",
  Mutation = "Mutation",
}

export class Query {
  name: string
  arguments: QueryArgument[]
  returns: OutputType
  resolver: string
  type: QueryType

  constructor(
    name: string,
    type: QueryType,
    returnType: OutputType,
    resolverName: string,
  ) {
    this.name = name
    this.arguments = []
    this.returns = returnType
    this.resolver = resolverName
    this.type = type
  }

  public pushArgument(name: string, type: InputType): Query {
    this.arguments.push(new QueryArgument(name, type))

    return this
  }
  
  public toString(): string {
    let header = `extend type ${this.type} {`
    let args = this.arguments.map(String).join(", ")
    let query = `  ${this.name}(${args}): ${this.returns} @resolver(name: "${this.resolver}")`
    let footer = "}"

    return `${header}\n${query}\n${footer}`
  }
}