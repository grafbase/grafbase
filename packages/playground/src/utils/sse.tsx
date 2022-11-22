import type { Fetcher, FetcherParams, FetcherResult } from '@graphiql/toolkit'
import { isLiveQueryOperationDefinitionNode } from '@n1ru4l/graphql-live-query'
import { applyLiveQueryJSONPatch } from '@n1ru4l/graphql-live-query-patch-json-patch'
import {
  applyAsyncIterableIteratorToSink,
  makeAsyncIterableIteratorFromSink
} from '@n1ru4l/push-pull-async-iterable-iterator'
import { Repeater } from '@repeaterjs/repeater'
import { DocumentNode, getOperationAST } from 'graphql'
import { createContext, ReactNode, useContext, useState } from 'react'
import ReconnectingEventSource from 'reconnecting-eventsource'

type ExecutionResult = Extract<FetcherResult, { hasNext?: boolean }>
type SSEClient = (url: string) => Repeater<ExecutionResult, any, unknown>

export const isLiveQuery = (document: DocumentNode, name: string | undefined): boolean => {
  const documentNode = getOperationAST(document, name)
  return !!documentNode ? isLiveQueryOperationDefinitionNode(documentNode) : false
}

type FetcherOptions = {
  url: string
  headers?: Record<string, string> | undefined
}

type SSEFetcherOptions = FetcherOptions & {
  sseClient: SSEClient
}

export const getSseFetcher = (options: SSEFetcherOptions) => {
  return (graphQLParams: FetcherParams) => {
    const headers = Object.entries(options.headers || {}).reduce((accumulator, [key, value]) => {
      if (value === undefined) {
        return accumulator
      }
      return {
        ...accumulator,
        [key]: value
      }
    }, {} as { [key: string]: any })
    const searchParams = new URLSearchParams({
      ...headers,
      query: graphQLParams.query,
      operationName: graphQLParams.operationName || '',
      variables: JSON.stringify(graphQLParams.variables || {})
    })
    const url = `${options.url}?${searchParams.toString()}`
    const sseClient = options.sseClient(url)

    return makeAsyncIterableIteratorFromSink<ExecutionResult>((sink) =>
      applyAsyncIterableIteratorToSink(sseClient, {
        ...sink
      })
    )
  }
}

type Status = 'OPEN' | 'CONNECTING' | 'CLOSED'

type SSEFetcherType = (options: FetcherOptions) => Fetcher

type SSEContextType = {
  sseFetcher: SSEFetcherType
  status: Status
}

const SSEContext = createContext<SSEContextType | null>(null)
SSEContext.displayName = 'SSEContext'

export const SSEProvider = ({ children }: { children?: ReactNode }) => {
  const [status, setStatus] = useState<Status>('CLOSED')

  /**
   * sseClient can be used as a base for our own lib to handle the sse connection.
   * @notrab suggested to create a lib to help users to adopt live queries in their apps and create plugins for Apollo, Relay, urql, etc.
   */
  const sseClient = (url: string) => {
    setStatus('CONNECTING')
    const eventSource = new ReconnectingEventSource(url)
    return applyLiveQueryJSONPatch(
      new Repeater<ExecutionResult>(async (push, end) => {
        eventSource.onmessage = (event) => {
          setStatus('OPEN')
          const data = JSON.parse(event.data)
          push(data)
          if (eventSource.readyState === 2) {
            end()
          }
        }
        eventSource.onerror = (error) => {
          end(error)
        }
        await end
        eventSource.close()
        setStatus('CLOSED')
      })
    )
  }

  const sseFetcher: SSEFetcherType = ({ url, headers }) => getSseFetcher({ url, headers, sseClient })

  return <SSEContext.Provider value={{ sseFetcher, status }}>{children}</SSEContext.Provider>
}

export const useSSEContext = () => {
  const context = useContext(SSEContext)
  if (!context) {
    throw new Error('SSEContext not found')
  }
  return context
}
