import { CreateFetcherOptions, createGraphiQLFetcher } from '@graphiql/toolkit'

type Options = Omit<CreateFetcherOptions, 'url'>

export const fetcher = (url: string, options: Options) => {
  return createGraphiQLFetcher({
    ...options,
    url
  })
}
