import {
  cacheExchange,
  createClient,
  dedupExchange,
  fetchExchange,
  makeOperation
} from 'urql'
import { authExchange as coreAuthExchange } from '@urql/exchange-auth'
import { sseExchange } from '@grafbase/urql-exchange'

const GRAFBASE_API_URL = import.meta.env.VITE_GRAFBASE_API_URL

const authExchange = coreAuthExchange<string>({
  addAuthToOperation: ({ authState, operation }) => {
    if (!authState) {
      return operation
    }
    return makeOperation(operation.kind, operation, {
      ...operation.context,
      fetchOptions: {
        headers: {
          authorization: `Bearer ${authState}`
        }
      }
    })
  },
  willAuthError: ({ authState }) => {
    if (!authState) return true
    return false
  },
  didAuthError: ({ error }) => {
    return error.graphQLErrors.some((e) => e.message.startsWith('Unauthorized'))
  },
  getAuth: async ({ authState, mutate }) => {
    if (typeof window === 'undefined') {
      return null
    }
    return localStorage.getItem('token') ?? ''
  }
})

export const client = createClient({
  url: GRAFBASE_API_URL,
  exchanges: [
    dedupExchange,
    cacheExchange,
    authExchange,
    sseExchange,
    fetchExchange
  ]
})
