import { GraphQLClient } from 'graphql-request'

export const graphQlRequestClient = new GraphQLClient(
  'http://localhost:3000/api/graphql'
)
