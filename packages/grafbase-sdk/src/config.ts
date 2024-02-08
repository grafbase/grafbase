import { AuthParams, AuthParamsV2, Authentication, AuthenticationV2 } from './auth'
import { CacheParams, GlobalCache } from './cache'
import { FederatedGraph, Graph } from './grafbase-schema'
import { OperationLimits, OperationLimitsParams } from './operation-limits'
import { Experimental, ExperimentalParams } from './experimental'

/**
 * An interface to create the complete config definition.
 */
export interface GraphConfigInput {
  graph: Graph
  auth?: AuthParams
  cache?: CacheParams
  operationLimits?: OperationLimitsParams
  experimental?: ExperimentalParams
}

/**
 * @deprecated use `graph` instead of `schema`
 * An interface to create the complete config definition.
 */
export interface DeprecatedGraphConfigInput {
  /** @deprecated use `graph` instead */
  schema: Graph
  auth?: AuthParams
  operationLimits?: OperationLimitsParams
  cache?: CacheParams
  experimental?: ExperimentalParams
}

/**
 * An interface to create the federation config definition.
 */
export interface FederatedGraphConfigInput {
  graph: FederatedGraph
  auth?: AuthParamsV2
  operationLimits?: OperationLimitsParams
}

/**
 * Defines the complete Grafbase configuration.
 */
export class GraphConfig {
  private graph: Graph
  private readonly auth?: Authentication
  private readonly cache?: GlobalCache
  private readonly operationLimits?: OperationLimits
  private readonly experimental?: Experimental

  /** @deprecated use `graph` instead of `schema` */
  constructor(input: GraphConfigInput | DeprecatedGraphConfigInput) {
    this.graph = 'graph' in input ? input.graph : input.schema

    if (input.auth) {
      this.auth = new Authentication(input.auth)
    }

    if (input.operationLimits) {
      this.operationLimits = new OperationLimits(input.operationLimits)
    }

    if (input.cache) {
      this.cache = new GlobalCache(input.cache)
    }

    if (input.experimental) {
      this.experimental = new Experimental(input.experimental)
    }
  }

  public toString(): string {
    const graph = this.graph.toString()
    const auth = this.auth ? this.auth.toString() : ''
    const operationLimits = this.operationLimits ? this.operationLimits.toString() : ''
    const cache = this.cache ? this.cache.toString() : ''
    const experimental = this.experimental ? this.experimental.toString() : ''

    return `${experimental}${auth}${operationLimits}${cache}${graph}`
  }
}

export class FederatedGraphConfig {
  private graph: FederatedGraph
  private readonly operationLimits?: OperationLimits
  private readonly auth?: AuthenticationV2

  constructor(input: FederatedGraphConfigInput) {
    this.graph = input.graph
    if (input.auth) {
      this.auth = new AuthenticationV2(input.auth)
    }
    if (input.operationLimits) {
      this.operationLimits = new OperationLimits(input.operationLimits)
    }
  }

  public toString(): string {
    const graph = this.graph.toString()
    const auth = this.auth ? this.auth.toString() : ''
    const operationLimits = this.operationLimits ? this.operationLimits.toString() : ''

    return `${auth}${graph}${operationLimits}`
  }
}
