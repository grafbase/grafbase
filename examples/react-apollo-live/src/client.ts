import {
  ApolloClient,
  ApolloLink,
  HttpLink,
  InMemoryCache,
  split
} from '@apollo/client'
import { isLiveQuery, SSELink } from '@grafbase/apollo-link'
import { getOperationAST } from 'graphql'

const GRAFBASE_API_URL = import.meta.env.VITE_GRAFBASE_API_URL

export const createApolloLink = (token: string) => {
  const sseLink = new SSELink({
    uri: GRAFBASE_API_URL,
    headers: {
      authorization: `Bearer ${token}`
    }
  })

  const httpLink = new HttpLink({
    uri: GRAFBASE_API_URL,
    headers: {
      authorization: `Bearer ${token}`
    }
  })

  return split(
    ({ query, operationName, variables }) =>
      isLiveQuery(getOperationAST(query, operationName), variables),
    sseLink,
    httpLink
  )
}

export const initializeApolloClient = (link: ApolloLink) => {
  return new ApolloClient({
    cache: new InMemoryCache(),
    link: link
  })
}
