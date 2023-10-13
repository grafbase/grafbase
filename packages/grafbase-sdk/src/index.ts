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
import { validateIdentifier } from './validation'
import { PostgresParams, PartialPostgresAPI } from './connector/postgres'

export { type ResolverContext as Context } from './resolver/context'
export { type ResolverInfo as Info } from './resolver/info'
export { type VerifiedIdentity } from './authorizer/verifiedIdentity'
export { type AuthorizerContext } from './authorizer/context'

dotenv.config({
  // must exist, defined by "~/.grafbase/parser/parse-config.ts"
  path: path.join(process.env.GRAFBASE_PROJECT_GRAFBASE_DIR!, '.env'),
  override: true,
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
   * @param name - A unique name for the connector.
   * @param params - The configuration parameters.
   */
  OpenAPI: (name: string, params: OpenAPIParams): PartialOpenAPI => {
    validateIdentifier(name)

    return new PartialOpenAPI(name, params)
  },
  /**
   * Create a new GraphQL connector object.
   *
   * @param name - A unique name for the connector.
   * @param params - The configuration parameters.
   */
  GraphQL: (name: string, params: GraphQLParams): PartialGraphQLAPI => {
    validateIdentifier(name)

    return new PartialGraphQLAPI(name, params)
  },
  /**
   * Create a new MongoDB connector object.
   *
   * @param name - A unique name for the connector.
   * @param params - The configuration parameters.
   */
  MongoDB: (name: string, params: MongoDBParams): PartialMongoDBAPI => {
    validateIdentifier(name)

    return new PartialMongoDBAPI(name, params)
  },
  /**
   * Create a new Postgres connector object.
   *
   * @param name - A unique name for the connector.
   * @param params - The configuration parameters.
   */
  Postgres: (name: string, params: PostgresParams): PartialPostgresAPI => {
    validateIdentifier(name)

    return new PartialPostgresAPI(name, params)
  },
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
  JWKS: (params: RequireAtLeastOne<JWKSParams, 'issuer' | 'jwksEndpoint'>): JWKSAuth => {
    return new JWKSAuth(params)
  },
  /**
   * Create a new authorizer authenticator.
   *
   * @param params - The configuration parameters.
   */
  Authorizer: (params: AuthorizerParams): Authorizer => {
    return new Authorizer(params)
  },
}
