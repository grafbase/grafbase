import { GraphQLClient } from 'graphql-request'

export const graphQlRequestClient = new GraphQLClient(
  process.env.NEXT_PUBLIC_GRAFBASE_API_URL!,
  {
    headers: {
      'Content-Type': 'application/json',
      'x-api-key': process.env.GRAFBASE_API_KEY!
    }
  }
)
