import { createGraphiQLFetcher, CreateFetcherOptions, Fetcher } from '@graphiql/toolkit'
import { buildClientSchema, getIntrospectionQuery } from 'graphql'

type Options = Omit<CreateFetcherOptions, 'url'>

export const fetcher = (url: string, options: Options) => {
  return createGraphiQLFetcher({
    ...options,
    url
  })
}

export const getSchema = async (fetcher: Fetcher) => {
  try {
    const response: any = await fetcher({
      query: getIntrospectionQuery(),
      operationName: 'IntrospectionQuery'
    })
    if (response?.data) {
      return buildClientSchema(response.data)
    }
  } catch (error) {
    console.error(error)
    const message = (error as any)?.message ?? error ?? 'Error fetching schema'
    throw new Error(message)
  }
}
