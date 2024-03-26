import {
  AuthParams,
  AuthParamsV2,
  Authentication,
  AuthenticationV2
} from './auth'
import { CacheParams, GlobalCache } from './cache'
import { FederatedGraph, Graph } from './grafbase-schema'
import { OperationLimits, OperationLimitsParams } from './operation-limits'
import { Experimental, ExperimentalParams } from './experimental'
import { Introspection } from './introspection'
import { TrustedDocuments, TrustedDocumentsParams } from './trusted-documents'

/**
 * An interface to create the complete config definition.
 */
export interface GraphConfigInput {
  graph: Graph
  auth?: AuthParams
  cache?: CacheParams
  operationLimits?: OperationLimitsParams
  trustedDocuments?: TrustedDocumentsParams
  experimental?: ExperimentalParams
  introspection?: boolean
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
  introspection?: boolean
  trustedDocuments?: TrustedDocumentsParams
}

/**
 * An interface to create the federation config definition.
 */
export interface FederatedGraphConfigInput {
  graph: FederatedGraph
  auth?: AuthParamsV2
  operationLimits?: OperationLimitsParams
  introspection?: boolean
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
  private readonly introspection?: Introspection
  private readonly trustedDocuments?: TrustedDocuments

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

    if (input.trustedDocuments) {
      this.trustedDocuments = new TrustedDocuments(input.trustedDocuments)
    }

    if (input.experimental) {
      this.experimental = new Experimental(input.experimental)
    }
    if (input.introspection !== undefined) {
      this.introspection = new Introspection({ enabled: input.introspection })
    }
  }

  public toString(): string {
    const graph = this.graph.toString()
    const auth = this.auth ? this.auth.toString() : ''
    const operationLimits = this.operationLimits
      ? this.operationLimits.toString()
      : ''
    const trustedDocuments = this.trustedDocuments
      ? this.trustedDocuments.toString()
      : ''
    const cache = this.cache ? this.cache.toString() : ''
    const experimental = this.experimental ? this.experimental.toString() : ''
    const introspection = this.introspection
      ? this.introspection.toString()
      : process.env.GRAFBASE_ENV === 'dev'
        ? new Introspection({ enabled: true })
        : ''

    return `${experimental}${auth}${operationLimits}${trustedDocuments}${cache}${introspection}${graph}`
  }
}

export class FederatedGraphConfig {
  private graph: FederatedGraph
  private readonly operationLimits?: OperationLimits
  private readonly auth?: AuthenticationV2
  private readonly introspection?: Introspection

  constructor(input: FederatedGraphConfigInput) {
    this.graph = input.graph
    if (input.auth) {
      this.auth = new AuthenticationV2(input.auth)
    }
    if (input.operationLimits) {
      this.operationLimits = new OperationLimits(input.operationLimits)
    }
    if (input.introspection !== undefined) {
      this.introspection = new Introspection({ enabled: input.introspection })
    }
  }

  public toString(): string {
    const graph = this.graph.toString()
    const auth = this.auth ? this.auth.toString() : ''
    const operationLimits = this.operationLimits
      ? this.operationLimits.toString()
      : ''

    const introspection = this.introspection
      ? this.introspection.toString()
      : process.env.GRAFBASE_ENV === 'dev'
        ? new Introspection({ enabled: true })
        : ''

    return `${auth}${graph}${operationLimits}${introspection}`
  }
}
