import { AuthParams, Authentication } from './auth'
import { GrafbaseSchema } from './grafbase-schema'

export interface ConfigInput {
  schema: GrafbaseSchema
  auth?: AuthParams
}

export class Config {
  schema: GrafbaseSchema
  auth?: Authentication

  constructor(input: ConfigInput) {
    this.schema = input.schema

    if (input.auth) {
      this.auth = new Authentication(input.auth)
    }
  }

  public toString(): string {
    const schema = this.schema.toString()
    const auth = this.auth ? this.auth.toString() : ""

    return `${auth}${schema}`
  }
}
