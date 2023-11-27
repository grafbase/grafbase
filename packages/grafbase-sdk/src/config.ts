import { AuthParams, Authentication } from './auth'
import { CacheParams, GlobalCache } from './cache'
import { FederatedGraph, SingleGraph } from './grafbase-schema'
import { Experimental, ExperimentalParams } from './experimental'
import { Federation, FederationParams } from './federation'

/**
 * DO NOT USE - experimental
 *
 * An interface to create the complete config definition.
 */
export interface ExperimentalSingleGraphConfigInput {
  graph: SingleGraph
  auth?: AuthParams
  cache?: CacheParams
  experimental?: ExperimentalParams
  federation?: FederationParams
}

// /**
//  * @deprecated use `graph` instead of `schema`
//  * An interface to create the complete config definition.
//  */
/*
 *
 * An interface to create the complete config definition.
 */
export interface SingleGraphConfigInput {
  // /** @deprecated use `graph` instead */
  schema: SingleGraph
  auth?: AuthParams
  cache?: CacheParams
  experimental?: ExperimentalParams
  federation?: FederationParams
}

/**
 * DO NOT USE - experimental
 *
 * An interface to create the federation config definition.
 */
export interface FederatedGraphConfigInput {
  graph: FederatedGraph
}

/**
 * Defines the complete Grafbase configuration.
 */
export class SingleGraphConfig {
  private graph: SingleGraph
  private readonly auth?: Authentication
  private readonly cache?: GlobalCache
  private readonly experimental?: Experimental
  private readonly federation?: Federation

  constructor(input: ExperimentalSingleGraphConfigInput)
  // /** @deprecated use `graph` instead of `schema` */
  constructor(input: SingleGraphConfigInput)
  constructor(
    input: ExperimentalSingleGraphConfigInput | SingleGraphConfigInput
  ) {
    this.graph = 'graph' in input ? input.graph : input.schema

    if (input.auth) {
      this.auth = new Authentication(input.auth)
    }

    if (input.cache) {
      this.cache = new GlobalCache(input.cache)
    }

    if (input.experimental) {
      this.experimental = new Experimental(input.experimental)
    }

    if (input.federation) {
      this.federation = new Federation(input.federation)
    }
  }

  public toString(): string {
    this.graph.federation = this.federation
    const graph = this.graph
    const auth = this.auth ? this.auth.toString() : ''
    const cache = this.cache ? this.cache.toString() : ''
    const experimental = this.experimental ? this.experimental.toString() : ''

    return `${experimental}${auth}${cache}${graph}`
  }
}

export class FederatedGraphConfig {
  private graph: FederatedGraph

  constructor(input: FederatedGraphConfigInput) {
    this.graph = input.graph
  }

  public toString(): string {
    return this.graph.toString()
  }
}
