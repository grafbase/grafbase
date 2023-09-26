import { AuthParams, Authentication } from './auth'
import { CacheParams, GlobalCache } from './cache'
import { GrafbaseSchema } from './grafbase-schema'
import { Experimental, ExperimentalParams } from './experimental'

/**
 * An interface to create the complete config definition.
 */
export interface ConfigInput {
  schema: GrafbaseSchema
  auth?: AuthParams
  cache?: CacheParams
  experimental?: ExperimentalParams
}

/**
 * Defines the complete Grafbase configuration.
 */
export class Config {
  private schema: GrafbaseSchema
  private readonly auth?: Authentication
  private readonly cache?: GlobalCache
  private readonly experimental?: Experimental

  constructor(input: ConfigInput) {
    this.schema = input.schema

    if (input.auth) {
      this.auth = new Authentication(input.auth)
    }

    if (input.cache) {
      this.cache = new GlobalCache(input.cache)
    }

    if (input.experimental) {
      this.experimental = new Experimental(input.experimental)
    }
  }

  public toString(): string {
    const schema = this.schema.toString()
    const auth = this.auth ? this.auth.toString() : ''
    const cache = this.cache ? this.cache.toString() : ''
    const experimental = this.experimental ? this.experimental.toString() : ''

    return `${experimental}${auth}${cache}${schema}`
  }
}
