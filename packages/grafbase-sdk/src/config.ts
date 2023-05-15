import { GrafbaseSchema } from './grafbase-schema'

export interface ConfigInput {
  schema: GrafbaseSchema
}

export class Config {
  schema: GrafbaseSchema

  constructor(input: ConfigInput) {
    this.schema = input.schema
  }

  public toString(): string {
    return this.schema.toString()
  }
}
