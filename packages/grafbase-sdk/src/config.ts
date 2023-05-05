import { Enum } from './enum';
import { Interface } from './interface';
import { Model } from './model'
import { Query } from './query';
import { Type } from './type';
import { Union } from './union';

export interface Schema {
  enums?: Enum[]
  types?: Type[]
  unions?: Union[]
  models?: Model[]
  interfaces?: Interface[]
  queries?: Query[]
}

export class Config {
  graphqlSchema?: Schema;

  public schema(schema: Schema): Config {
    this.graphqlSchema = schema

    return this
  }

  public toString(): string {
    const queries = this.graphqlSchema?.queries?.map(String).join("\n\n") ?? ""
    const interfaces = this.graphqlSchema?.interfaces?.map(String).join("\n\n") ?? ""
    const types = this.graphqlSchema?.types?.map(String).join("\n\n") ?? ""
    const unions = this.graphqlSchema?.unions?.map(String).join("\n\n") ?? ""
    const enums = this.graphqlSchema?.enums?.map(String).join("\n\n") ?? ""
    const models = this.graphqlSchema?.models?.map(String).join("\n\n") ?? ""

    const renderOrder = [interfaces, enums, types, queries, unions, models]

    return renderOrder.filter(Boolean).flat().map(String).join("\n\n")
  }
}
