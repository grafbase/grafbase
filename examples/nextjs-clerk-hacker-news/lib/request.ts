import { GraphQLClient } from 'graphql-request'

export const graphQlRequestClient = new GraphQLClient(
  process.env.NEXT_PUBLIC_GRAFBASE_API_URL!
)
