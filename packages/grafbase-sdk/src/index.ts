import { GrafbaseSchema } from './grafbase-schema'
import { Config, ConfigInput } from './config'
import { OpenAPIParams, PartialOpenAPI } from './connector/openapi'
import { GraphQLParams, PartialGraphQLAPI } from './connector/graphql'
import { OpenIDAuth, OpenIDParams } from './auth/openid'
import { JWTAuth, JWTParams } from './auth/jwt'
import { JWKSAuth, JWKSParams } from './auth/jwks'
import { RequireAtLeastOne } from 'type-fest'

export type AtLeastOne<T> = [T, ...T[]]

/**
 * A builder for a Grafbase schema definition.
 */
export const g = new GrafbaseSchema()

/**
 * A constructor for a complete Grafbase configuration.
 */
export function config(input: ConfigInput): Config {
  return new Config(input)
}

export const connector = {
  /**
   * Create a new OpenAPI connector object.
   */
  OpenAPI: (params: OpenAPIParams): PartialOpenAPI => {
    return new PartialOpenAPI(params)
  },
  /**
   * Create a new GraphQL connector object.
   */
  GraphQL: (params: GraphQLParams): PartialGraphQLAPI => {
    return new PartialGraphQLAPI(params)
  }
}

export const auth = {
  /**
   * Create a new OpenID authenticator.
   */
  OpenIDConnect: (params: OpenIDParams): OpenIDAuth => {
    return new OpenIDAuth(params)
  },
  /**
   * Create a new JWT authenticator.
   */
  JWT: (params: JWTParams): JWTAuth => {
    return new JWTAuth(params)
  },
  /**
   * Create a new JWKS authenticator.
   */
  JWKS: (
    params: RequireAtLeastOne<JWKSParams, 'issuer' | 'jwksEndpoint'>
  ): JWKSAuth => {
    return new JWKSAuth(params)
  }
}
