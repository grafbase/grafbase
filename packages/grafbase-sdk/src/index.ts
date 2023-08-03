import { GrafbaseSchema } from './grafbase-schema'
import { Config, ConfigInput } from './config'
import { OpenAPIParams, PartialOpenAPI } from './connector/openapi'
import { GraphQLParams, PartialGraphQLAPI } from './connector/graphql'
import { OpenIDAuth, OpenIDParams } from './auth/openid'
import { JWTAuth, JWTParams } from './auth/jwt'
import { JWKSAuth, JWKSParams } from './auth/jwks'
import { RequireAtLeastOne } from 'type-fest'
import dotenv from 'dotenv'
import { Authorizer, AuthorizerParams } from './auth/authorizer'
import { MongoDBParams, PartialMongoDBAPI } from './connector/mongodb'
import path from 'path'

dotenv.config({
  // must exist, defined by "~/.grafbase/parser/parse-config.ts"
  path: path.join(process.env.GRAFBASE_PROJECT_GRAFBASE_DIR!, '.env')
})

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
   *
   * @param params - The configuration parameters.
   */
  OpenAPI: (params: OpenAPIParams): PartialOpenAPI => {
    return new PartialOpenAPI(params)
  },
  /**
   * Create a new GraphQL connector object.
   *
   * @param params - The configuration parameters.
   */
  GraphQL: (params: GraphQLParams): PartialGraphQLAPI => {
    return new PartialGraphQLAPI(params)
  },
  /**
   * Create a new MongoDB connector object.
   *
   * @param params = The configuration parameters.
   */
  MongoDB: (params: MongoDBParams): PartialMongoDBAPI => {
    return new PartialMongoDBAPI(params)
  }
}

export const auth = {
  /**
   * Create a new OpenID authenticator.
   *
   * @param params - The configuration parameters.
   */
  OpenIDConnect: (params: OpenIDParams): OpenIDAuth => {
    return new OpenIDAuth(params)
  },
  /**
   * Create a new JWT authenticator.
   *
   * @param params - The configuration parameters.
   */
  JWT: (params: JWTParams): JWTAuth => {
    return new JWTAuth(params)
  },
  /**
   * Create a new JWKS authenticator.
   *
   * @param params - The configuration parameters.
   */
  JWKS: (
    params: RequireAtLeastOne<JWKSParams, 'issuer' | 'jwksEndpoint'>
  ): JWKSAuth => {
    return new JWKSAuth(params)
  },
  /**
   * Create a new authorizer authenticator.
   *
   * @param params - The configuration parameters.
   */
  Authorizer: (params: AuthorizerParams): Authorizer => {
    return new Authorizer(params)
  }
}
