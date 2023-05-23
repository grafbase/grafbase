import { GrafbaseSchema } from './grafbase-schema'
import { Config, ConfigInput } from './config'
import { OpenAPIParams, PartialOpenAPI } from './connector/openapi'
import { GraphQLParams, PartialGraphQLAPI } from './connector/graphql'
import { OpenIDAuth, OpenIDParams } from './auth/openid'
import { JWTAuth, JWTParams } from './auth/jwt'
import { JWKSAuth, JWKSParams } from './auth/jwks'
import { RequireExactlyOne } from 'type-fest'

export type AtLeastOne<T> = [T, ...T[]]

export const g = new GrafbaseSchema()

export function config(input: ConfigInput): Config {
  return new Config(input)
}

export const connector = {
  OpenAPI: (params: OpenAPIParams): PartialOpenAPI => {
    return new PartialOpenAPI(params)
  },
  GraphQL: (params: GraphQLParams): PartialGraphQLAPI => {
    return new PartialGraphQLAPI(params)
  }
}

export const auth = {
  OpenIDConnect: (params: OpenIDParams): OpenIDAuth => {
    return new OpenIDAuth(params)
  },
  JWT: (params: JWTParams): JWTAuth => {
    return new JWTAuth(params)
  },
  JWKS: (
    params: RequireExactlyOne<JWKSParams, 'issuer' | 'jwksEndpoint'>
  ): JWKSAuth => {
    return new JWKSAuth(params)
  }
}
