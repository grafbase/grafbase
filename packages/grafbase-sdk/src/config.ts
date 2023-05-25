import { AuthParams, Authentication } from './auth'
import { CacheParams, GlobalCache } from './cache'
import { GrafbaseSchema } from './grafbase-schema'

/**
 * An interface to create the complete config definition.
 */
export interface ConfigInput {
  schema: GrafbaseSchema
  auth?: AuthParams
  cache?: CacheParams
}

/**
 * Defines the complete Grafbase configuration.
 */
export class Config {
  schema: GrafbaseSchema
  auth?: Authentication
  cache?: GlobalCache

  constructor(input: ConfigInput) {
    this.schema = input.schema

    if (input.auth) {
      this.auth = new Authentication(input.auth)
    }

    if (input.cache) {
      this.cache = new GlobalCache(input.cache)
    }
  }

  public toString(): string {
    const schema = this.schema.toString()
    const auth = this.auth ? this.auth.toString() : ''
    const cache = this.cache ? this.cache.toString() : ''

    return `${auth}${cache}${schema}`
  }
}
