import {
  ApolloClient,
  ApolloProvider,
  InMemoryCache,
  from,
  HttpLink
} from '@apollo/client'
import { setContext } from '@apollo/client/link/context'
import { relayStylePagination } from '@apollo/client/utilities'
import { useAuth } from '@clerk/nextjs'
import type { PropsWithChildren } from 'react'
import { useMemo } from 'react'

const httpLink = new HttpLink({
  uri: process.env.NEXT_PUBLIC_GRAFBASE_API_URL
})

const ApolloProviderWrapper = ({ children }: PropsWithChildren) => {
  const { getToken } = useAuth()

  const client = useMemo(() => {
    const authMiddleware = setContext(async (operation, { headers }) => {
      const token = await getToken({ template: 'grafbase' })

      return {
        headers: {
          ...headers,
          authorization: `Bearer ${token}`
        }
      }
    })

    return new ApolloClient({
      link: from([authMiddleware, httpLink]),
      cache: new InMemoryCache({
        typePolicies: {
          Item: {
            fields: {
              comments: relayStylePagination()
            }
          },
          Query: {
            fields: {
              itemCollection: relayStylePagination(),
              userCollection: relayStylePagination()
            }
          }
        }
      })
    })
  }, [getToken])

  return <ApolloProvider client={client}>{children}</ApolloProvider>
}

export default ApolloProviderWrapper
